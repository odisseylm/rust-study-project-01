
insert into USERS (ID, NAME, PASSWORD, PSW_HASH) values (1, 'client1', 'psw1', '$argon2d$v=16$m=19456,t=2,p=1$6suBJobcNzc8KE/leXzHKQ$etGYL8dLVSIFlX7TyclQczqfmx4hPm64nV3819H4vEo');
insert into USERS (ID, NAME, PASSWORD, PSW_HASH) values (2, 'client2', 'psw2', '$argon2d$v=16$m=19456,t=2,p=1$lJ4IasWc2DTlqUobU0IWGw$ef49CpVe7Id4Sg2P4Ly7UdYkwlih5H8CsPy0ITzwLiM');
insert into USERS (ID, NAME, PASSWORD, PSW_HASH) values (3, 'client3', 'psw3', '$argon2d$v=16$m=19456,t=2,p=1$GjKFhhuGE1wDSpPwPCVIJQ$wOZVOOvDFYTyhjCtRXYr3X0i6IFvseKQNtjHYrgNCQA');
insert into USERS (ID, NAME, PASSWORD, PSW_HASH) values (4, 'client4', 'psw4', '$argon2d$v=16$m=19456,t=2,p=1$n3UFN93PKuf2OrK2xyXs+w$0pt4vmIHK3bWycKXJW6IWkQ/P9WWikpCrol5gcvll1k');

insert into USERS (ID, NAME, PASSWORD, PSW_HASH) values (21, 'vovan', 'qwerty', '$argon2d$v=16$m=19456,t=2,p=1$66zW697+kjcdOfLDOs6GGA$76kr0gFz1xQ6o2bpTqwoetOn0RoJ7QrSxRoMwLkA0xg');
insert into USERS (ID, NAME, PASSWORD, PSW_HASH) values (22, 'vovan-read', 'qwerty', '$argon2d$v=16$m=19456,t=2,p=1$1jd9ljbST1E/CsGOiBOsOg$4NUBW5Rf7Cm/E/+YtoQw2Vg3270pO5EWufqo3qmS7oE');
insert into USERS (ID, NAME, PASSWORD, PSW_HASH) values (23, 'vovan-write', 'qwerty', '$argon2d$v=16$m=19456,t=2,p=1$EUXO4jWOfnouAFF+CipWgQ$TleOX6tIVbCkynIGptjzRTcFVSbS2K3tZcmPckwJVt8');
insert into USERS (ID, NAME, PASSWORD, PSW_HASH) values (24, 'vovan-read-write', 'qwerty', '$argon2d$v=16$m=19456,t=2,p=1$66zW697+kjcdOfLDOs6GGA$76kr0gFz1xQ6o2bpTqwoetOn0RoJ7QrSxRoMwLkA0xg');
insert into USERS (ID, NAME, PASSWORD, PSW_HASH) values (25, 'vovan-user', 'qwerty', '$argon2d$v=16$m=19456,t=2,p=1$1jd9ljbST1E/CsGOiBOsOg$4NUBW5Rf7Cm/E/+YtoQw2Vg3270pO5EWufqo3qmS7oE');
insert into USERS (ID, NAME, PASSWORD, PSW_HASH) values (26, 'vovan-super-user', 'qwerty', '$argon2d$v=16$m=19456,t=2,p=1$EUXO4jWOfnouAFF+CipWgQ$TleOX6tIVbCkynIGptjzRTcFVSbS2K3tZcmPckwJVt8');
insert into USERS (ID, NAME, PASSWORD, PSW_HASH) values (27, 'vovan-admin', 'qwerty', '$argon2d$v=16$m=19456,t=2,p=1$66zW697+kjcdOfLDOs6GGA$76kr0gFz1xQ6o2bpTqwoetOn0RoJ7QrSxRoMwLkA0xg');


insert into USER_ROLES (USER_ID, READ_ROLE)  values (22, 'y');
insert into USER_ROLES (USER_ID, WRITE_ROLE) values (23, 'y');
insert into USER_ROLES (USER_ID, READ_ROLE, WRITE_ROLE) values (24, 'y', 'y');
--insert into USER_ROLES (USER_ID, USER_ROLE) values (25, 'y');
insert into USER_ROLES (USER_ID, SUPER_USER_ROLE) values (26, 'y');
insert into USER_ROLES (USER_ID, ADMIN_ROLE) values (27, 'y');
