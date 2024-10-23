-- Vérifier et ajouter token_sub_event_id si nécessaire
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'token_event'
        AND column_name = 'token_sub_event_id'
    ) THEN
        -- D'abord ajouter la colonne comme nullable
        ALTER TABLE token_event ADD COLUMN token_sub_event_id VARCHAR(78);
        
        -- Mettre à jour les valeurs NULL existantes avec une valeur par défaut
        UPDATE token_event SET token_sub_event_id = token_event_id WHERE token_sub_event_id IS NULL;
        
        -- Ensuite la rendre NOT NULL
        ALTER TABLE token_event ALTER COLUMN token_sub_event_id SET NOT NULL;
    END IF;
END $$;

-- 1. Supprimer l'ancienne contrainte
ALTER TABLE contract DROP CONSTRAINT IF EXISTS contract_contract_type_check;

-- 2. Créer le type enum et convertir la colonne
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'contract_type') THEN
        CREATE TYPE contract_type AS ENUM ('ERC721', 'ERC1155', 'ERC20', 'ERC1400','OTHER');
    END IF;
END$$;

-- 3. Convertir la colonne en enum
ALTER TABLE contract ALTER COLUMN contract_type TYPE contract_type USING contract_type::contract_type;

-- 4. Insérer les contrats manquants
WITH distinct_contracts AS (
    -- Dédupliquer d'abord les contrats
    SELECT DISTINCT ON (contract_address, chain_id)
        ti.contract_address,
        '0x534e5f4d41494e' as chain_id,
        CASE 
            WHEN ti.contract_type::text = 'ERC20' THEN 'ERC20'::contract_type
            WHEN ti.contract_type::text = 'ERC721' THEN 'ERC721'::contract_type
            WHEN ti.contract_type::text = 'ERC1155' THEN 'ERC1155'::contract_type
            WHEN ti.contract_type::text = 'ERC1400' THEN 'ERC1400'::contract_type
            ELSE 'OTHER'::contract_type
        END as contract_type
    FROM transaction_info ti
    LEFT JOIN contract c ON 
        c.contract_address = ti.contract_address AND 
        c.chain_id = '0x534e5f4d41494e'
    WHERE c.contract_address IS NULL
    ORDER BY contract_address, chain_id, ti.sequence_id DESC
)
INSERT INTO contract (
    contract_address,
    chain_id,
    contract_type,
    updated_timestamp,
    metadata_ok,
    is_spam,
    is_nsfw,
    is_verified,
    save_images
)
SELECT 
    contract_address,
    chain_id,
    contract_type,
    EXTRACT(epoch FROM now())::bigint as updated_timestamp,
    false as metadata_ok,
    false as is_spam,
    false as is_nsfw,
    false as is_verified,
    false as save_images
FROM distinct_contracts
ON CONFLICT (contract_address, chain_id) DO NOTHING;

-- 5. Insérer les tokens manquants
WITH source_tokens AS (
    SELECT 
        ti.contract_address,
        '0x534e5f4d41494e' as chain_id,
        COALESCE(CAST(ti.token_id as TEXT), '0') as token_id,
        CASE 
            WHEN ti.token_id IS NOT NULL THEN 
                '0x' || LPAD(
                    UPPER(
                        TRIM(
                            REGEXP_REPLACE(
                                CAST(ti.token_id as TEXT),
                                '[^0-9A-Fa-f]',
                                '',
                                'g'
                            )
                        )
                    ),
                    64,
                    '0'
                )
            ELSE '0x' || LPAD('0', 64, '0')
        END as token_id_hex,
        ti.timestamp as block_timestamp
    FROM transaction_info ti
    LEFT JOIN token t ON 
        t.contract_address = ti.contract_address AND
        t.chain_id = '0x534e5f4d41494e' AND
        t.token_id = COALESCE(CAST(ti.token_id as TEXT), '0')
    WHERE t.contract_address IS NULL
),
missing_tokens AS (
    -- Dédupliquer les tokens
    SELECT DISTINCT ON (contract_address, chain_id, token_id)
        contract_address,
        chain_id,
        token_id,
        token_id_hex,
        block_timestamp
    FROM source_tokens
    ORDER BY contract_address, chain_id, token_id, block_timestamp DESC
)
INSERT INTO token (
    contract_address,
    chain_id,
    token_id,
    token_id_hex,
    block_timestamp,
    buy_in_progress,
    has_bid,
    is_burned,
    metadata_status,
    updated_timestamp
)
SELECT 
    contract_address,
    chain_id,
    token_id,
    token_id_hex,
    block_timestamp,
    false as buy_in_progress,
    false as has_bid,
    false as is_burned,
    'TO_REFRESH' as metadata_status,
    EXTRACT(epoch FROM now())::bigint as updated_timestamp
FROM missing_tokens
ON CONFLICT (contract_address, chain_id, token_id) DO NOTHING;
-- 6. Migration vers token_event
TRUNCATE token_event RESTART IDENTITY CASCADE;

DO $$
DECLARE
    batch_size INT := 100000;
    total_rows INT;
    processed_rows INT := 0;
    current_batch INT := 0;
    error_count INT := 0;
    max_errors INT := 5;
BEGIN
    -- Créer une table temporaire pour stocker les données dédupliquées
    CREATE TEMP TABLE temp_token_events AS
    SELECT DISTINCT ON (event_id)
        event_id as token_event_id,
        contract_address,
        '0x534e5f4d41494e' as chain_id,
        COALESCE(CAST(token_id as TEXT), '0') as token_id,
        CASE 
            WHEN token_id IS NOT NULL THEN 
                '0x' || LPAD(
                    UPPER(
                        TRIM(
                            REGEXP_REPLACE(
                                CAST(token_id as TEXT),
                                '[^0-9A-Fa-f]',
                                '',
                                'g'
                            )
                        )
                    ),
                    64,
                    '0'
                )
            ELSE '0x' || LPAD('0', 64, '0')
        END as token_id_hex,
        CAST(event_type as TEXT) as event_type,
        timestamp as block_timestamp,
        tx_hash as transaction_hash,
        to_address,
        from_address,
        CAST(value as TEXT) as amount,
        sub_event_id as token_sub_event_id
    FROM transaction_info
    ORDER BY event_id, sequence_id DESC;

    SELECT COUNT(*) INTO total_rows FROM temp_token_events;
    RAISE NOTICE 'Nombre total de lignes à migrer (après déduplication) : %', total_rows;

    <<batch_loop>>
    WHILE current_batch * batch_size < total_rows LOOP
        BEGIN
            INSERT INTO token_event (
                token_event_id,
                contract_address,
                chain_id,
                token_id,
                token_id_hex,
                event_type,
                block_timestamp,
                transaction_hash,
                to_address,
                from_address,
                amount,
                token_sub_event_id
            )
            SELECT 
                token_event_id,
                contract_address,
                chain_id,
                token_id,
                token_id_hex,
                event_type,
                block_timestamp,
                transaction_hash,
                to_address,
                from_address,
                amount,
                token_sub_event_id
            FROM temp_token_events
            OFFSET current_batch * batch_size
            LIMIT batch_size;

            GET DIAGNOSTICS processed_rows = ROW_COUNT;
            current_batch := current_batch + 1;
            
            RAISE NOTICE 'Traité % lignes sur % (Lot %)', 
                LEAST(current_batch * batch_size, total_rows), 
                total_rows,
                current_batch;
            
        EXCEPTION 
            WHEN OTHERS THEN
                error_count := error_count + 1;
                RAISE WARNING 'Erreur survenue: %. Tentative de reprise...', SQLERRM;
                
                IF error_count >= max_errors THEN
                    RAISE EXCEPTION 'Nombre maximum d''erreurs atteint. Arrêt du processus.';
                END IF;
                
                PERFORM pg_sleep(5);
                CONTINUE batch_loop;
        END;
    END LOOP;

    -- Nettoyer la table temporaire
    DROP TABLE temp_token_events;
END $$;