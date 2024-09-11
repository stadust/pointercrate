-- Your SQL goes here

alter table members alter column password_hash type text using encode(password_hash, 'escape');
