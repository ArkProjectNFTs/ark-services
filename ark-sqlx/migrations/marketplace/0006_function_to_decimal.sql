CREATE OR REPLACE FUNCTION hex_to_decimal(hex_string text)
    RETURNS numeric
    LANGUAGE plpgsql IMMUTABLE AS $$
DECLARE
bits bit varying;
    result numeric := 0;
    exponent numeric := 0;
    chunk_size integer := 31;
    start integer;
BEGIN
    -- Return NULL if input is NULL
    IF hex_string IS NULL THEN
        RETURN NULL;
END IF;

    -- Remove '0x' prefix if it exists
    IF left(hex_string, 2) = '0x' THEN
        hex_string := substr(hex_string, 3);
END IF;

EXECUTE 'SELECT x' || quote_literal(hex_string) INTO bits;

WHILE length(bits) > 0 LOOP
        start := greatest(0, length(bits) - chunk_size) + 1;
        result := result + (substring(bits FROM start FOR chunk_size)::bigint)::numeric * pow(2::numeric, exponent);
        exponent := exponent + chunk_size;
        bits := substring(bits FROM 1 FOR greatest(0, length(bits) - chunk_size));
END LOOP;

RETURN result;
END
$$;
