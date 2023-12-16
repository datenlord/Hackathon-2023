time curl --location --request POST 'http://127.0.0.1:48452/api/v1/start_download' \
--header 'User-Agent: Apifox/1.0.0 (https://apifox.com)' \
--header 'Content-Type: application/json' \
--data-raw '{
    "filename": "output_10MB.txt"
}'