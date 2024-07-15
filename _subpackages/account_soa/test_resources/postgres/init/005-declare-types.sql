

-- See https://www.postgresql.org/docs/current/collation.html
CREATE COLLATION ENGLISH_CI (
   PROVIDER = icu, -- if the server was built with ICU support
   -- provider = libc,
   -- 'en-US@colStrength=secondary' for old ICU versions
   -- '@colStrength=secondary'
   locale = 'en-US-u-ks-level2',
   deterministic = false
);
---------------------------------------------------------------------------------

create domain US_POSTAL_CODE as TEXT
    check (
        value ~ '^\d{5}$'
     or value ~ '^\d{5}-\d{4}$'
);

create domain IBAN as VARCHAR(34)
    check (length(value) >= 16 and length(value) <= 34);

create domain ACCOUNT_ID as IBAN;

create domain AMOUNT as NUMERIC(15,6);

create domain CURRENCY as CHAR(3);
