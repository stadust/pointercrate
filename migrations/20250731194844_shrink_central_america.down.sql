-- Add down migration script here
UPDATE nationalities SET continent = 'Central America' WHERE nation = 'Mexico' OR nation = 'Cuba';