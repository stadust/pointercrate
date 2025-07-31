-- Add up migration script here
UPDATE nationalities SET continent = 'North America' WHERE nation = 'Mexico' OR nation = 'Cuba';