import fetch from 'node-fetch';

export async function exportToPostman(env: string, stage: string, postmanApiKey: string, restApiId: string, region: string) {
    try {
        // Fetch the OpenAPI definition from API Gateway
        const apiExportUrl = `https://apigateway.${region}.amazonaws.com/restapis/${restApiId}/stages/${stage}/exports/oas30?extensions=postman`;
        const apiResponse = await fetch(apiExportUrl, {
            headers: {
                'Accept': 'application/json'
            }
        });

        if (!apiResponse.ok) {
            throw new Error(`Failed to fetch API definition: ${await apiResponse.text()}`);
        }

        const apiDefinition = await apiResponse.json() as { paths: string[] };

        // Update Postman collection
        const collectionName = `ARK Project - ${env} - ${stage}`;
        const headers = {
            'X-Api-Key': postmanApiKey,
            'Content-Type': 'application/json'
        };

        const postmanResponse = await fetch('https://api.getpostman.com/collections', {
            method: 'POST',
            headers: headers,
            body: JSON.stringify({
                collection: {
                    info: {
                        name: collectionName,
                        description: `API Collection for ${env} environment on ${stage} stage.`,
                        schema: 'https://schema.getpostman.com/json/collection/v2.1.0/collection.json'
                    },
                    item: apiDefinition.paths // Convert the Swagger paths to Postman format
                }
            })
        });

        if (postmanResponse.ok) {
            console.log(`Successfully updated collection: ${collectionName}`);
        } else {
            console.error('Error updating Postman collection:', await postmanResponse.text());
        }
    } catch (error) {
        console.error('Error exporting API Gateway to Postman:', error);
    }
}
