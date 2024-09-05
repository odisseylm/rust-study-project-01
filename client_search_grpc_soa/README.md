

 - `grpcurl -plaintext localhost:8080 list`
 - `grpcurl -plaintext localhost:8080 describe grpc_health_v1.Health`
 - `grpcurl -d '{ "service": "" }' -plaintext localhost:8080 grpc_health_v1.Health/Check`
 - `grpcurl -d '{ "user_email": "cheburan@ukr.net" }' -plaintext localhost:8080 mvv_client_search_api_v1.ClientSearchService/Search`
 - `grpcurl -d '{ "user_email": "cheburan@ukr.net" }' -plaintext localhost:3002 mvv_client_search_api_v1.ClientSearchService/Search`
