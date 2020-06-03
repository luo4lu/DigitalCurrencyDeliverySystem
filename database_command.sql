create database DigitalCurrencyDeliverySystem;

create table digital_currency(
    id varchar(255) PRIMARY KEY NOT NULL,
    quota_control_field text NOT NULL,
    explain_info jsonb NOT NULL,
    state varchar(255) NOT NULL,
    owner text NOT NULL,
    create_time timestamp,
    update_time timestamp NOT NULL
);