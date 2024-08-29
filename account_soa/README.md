

 - `curl --verbose -X POST -H "Content-Type: application/json" -d '{"email_parent_filed":{"email33":"a@b@c"}}' http://localhost:3000/api/current_user/validate_test/input_validate_1`
 - Good
   - `curl --verbose -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" "http://localhost:3000/api/client/00000000-0000-0000-0000-000000000001/account/UA713736572172926969841832393" `
   - `curl --verbose -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" "http://localhost:3000/api/client/00000000-0000-0000-0000-000000000001/account/all" `
   - `curl --verbose --insecure -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" "https://localhost:3000/api/client/00000000-0000-0000-0000-000000000001/account/all" `
   - `curl --verbose --cert /home/vmelnykov/projects/rust/rust-study-project-01/target/generated-test-resources/ssl/rust-account-soa.crt.pem -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" "https://localhost:3000/api/client/00000000-0000-0000-0000-000000000001/account/all" `
   - `curl --verbose --insecure -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" "https://localhost:8443/api/client/00000000-0000-0000-0000-000000000001/account/all" `
   - `curl --verbose --insecure -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" "https://localhost:8101/api/client/00000000-0000-0000-0000-000000000001/account/all" `
   - Using cert
     - `curl --verbose --cacert ../target/generated-test-resources/ssl/rust-account-soa.crt.pem -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" "https://localhost:8101/api/client/00000000-0000-0000-0000-000000000001/account/all" `
 - Incorrect client ID and Iban
   - ValidifyErrors
     - `curl --verbose -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" "http://localhost:3000/api/client/00000000-0000-%D1%87%D0%B5%D0%B1%D1%83%D1%80%D0%B0%D0%BD/account/UA713736572172926969841832393" `
   - ValidationError (from domain object creation)
     - `curl --verbose -H "Authorization: Basic dm92YW4tcmVhZDpxd2VydHk=" "http://localhost:3000/api/client/00000000-00/account/UA713736572172926969841832393" `
