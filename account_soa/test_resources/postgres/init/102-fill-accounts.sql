
insert into CLIENTS (USER_ID, CLIENT_ID, EMAIL, PHONE)
values (101, '00000000-0000-0000-0000-000000000001', 'cheburan@ukr.net', '+380671234567');

--

insert into ACCOUNTS (ID, IBAN, CLIENT_ID, NAME, CREATED_AT, AMOUNT, CUR)
values ('00000000-0000-0000-0000-000000000101', 'UA713736572172926969841832393', '00000000-0000-0000-0000-000000000001',
 'USD account 1', TIMESTAMP '2021-11-10 15:14:13', 150.0, 'USD');

insert into ACCOUNTS (ID, IBAN, CLIENT_ID, NAME, CREATED_AT, AMOUNT, CUR)
values ('00000000-0000-0000-0000-000000000102', 'UA948614766857337364625464668', '00000000-0000-0000-0000-000000000001',
 'USD account 2', TIMESTAMP '2021-11-11 16:00:00', 250.0, 'USD');

insert into ACCOUNTS (ID, IBAN, CLIENT_ID, NAME, CREATED_AT, AMOUNT, CUR)
values ('00000000-0000-0000-0000-000000000103', 'UA565117374274826517545533479', '00000000-0000-0000-0000-000000000001',
 'UAH account 1', TIMESTAMP '2021-11-12 17:00:00', 1000.0, 'UAH');

insert into ACCOUNTS (ID, IBAN, CLIENT_ID, NAME, CREATED_AT, AMOUNT, CUR)
values ('00000000-0000-0000-0000-000000000104', 'UA496826153843944716538382928', '00000000-0000-0000-0000-000000000001',
 'UAH account 2', TIMESTAMP '2021-11-13 10:00:00', 2000.0, 'UAH');
