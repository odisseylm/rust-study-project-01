

 - `grpcurl -plaintext localhost:8080 list`
 - `grpcurl -plaintext localhost:8080 describe grpc.health.v1.Health`
 - `grpcurl -d '{ "service": "" }' -plaintext localhost:8080 grpc.health.v1.Health/Check`
 - `grpcurl -d '{ "user_email": "cheburan@ukr.net" }' -plaintext localhost:8080 mvv.client.search.api.v1.ClientSearchService/Search`
 - `grpcurl -d '{ "user_email": "cheburan@ukr.net" }' -plaintext localhost:3002 mvv.client.search.api.v1.ClientSearchService/Search`
 - With good user
   - `grpcurl -d '{ "user_email": "cheburan@ukr.net" }' -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" -plaintext localhost:3002 mvv.client.search.api.v1.ClientSearchService/Search`
 - With good user and wrong password
   - `grpcurl -d '{ "user_email": "cheburan@ukr.net" }' -H "Authorization: Basic dm92YW4td3JpdGU6cXdlcnR5NjY2" -plaintext localhost:3002 mvv.client.search.api.v1.ClientSearchService/Search`
 - With bad/weak user
   - TODO: impl authz, now only authentication is used
   - `grpcurl -d '{ "user_email": "cheburan@ukr.net" }' -H "Authorization: Basic dm92YW46cXdlcnR5" -plaintext localhost:3002 mvv.client.search.api.v1.ClientSearchService/Search`
 - `curl http://localhost:3002/healthcheck`
