curl --location --request GET 'localhost:8888/questions' \
 --header 'Content-Type: application/json'

curl --location --request GET 'localhost:8888/questions/1001' \
--header 'Content-Type: application/json'

curl --location --request PUT 'localhost:8888/questions/2' \
 --header 'Content-Type: application/json' \
--data-raw '{
"id": "2",
"title": "NEW TITLE", "content": "OLD CONTENT"
}'

curl --location --request POST 'localhost:8888/questions' \
 --header 'Content-Type: application/json' \
--data-raw '{
"id": "10",
"title": "NEW TITLE", "content": "OLD CONTENT"
}'


