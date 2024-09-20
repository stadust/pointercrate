CREATE TABLE subdivisions (
  iso_code VARCHAR(3),
  name CITEXT UNIQUE NOT NULL,
  nation VARCHAR(2) REFERENCES nationalities(iso_country_code),

  PRIMARY KEY (iso_code, nation)
);

CREATE TYPE continent AS ENUM ('Asia', 'Europe', 'Australia and Oceania', 'Africa', 'North America', 'South America', 'Central America');

ALTER TABLE nationalities ADD COLUMN continent continent;

INSERT INTO nationalities (iso_country_code, nation, continent)
VALUES
    ('BQ', 'Bonaire', 'Central America'),
    ('JM', 'Jamaica', 'Central America'), 
    ('PR', 'Puerto Rico', 'Central America'), 
    ('DO', 'Dominican Republic', 'Central America'), 
    ('HT', 'Haiti', 'Central America'), 
    ('SV', 'El Salvador', 'Central America'), 
    ('GT', 'Guatemala', 'Central America'), 
    ('HN', 'Honduras', 'Central America'), 
    ('NI', 'Nicaragua', 'Central America'), 
    ('PA', 'Panama', 'Central America'), 
    ('CR', 'Costa Rica', 'Central America'), 
    ('MX', 'Mexico', 'Central America'), 
    ('MS', 'Montserrat', 'Central America'), 
    ('VG', 'British Virgin Islands', 'Central America'), 
    ('VI', 'US Virgin Islands', 'Central America'), 
    ('KN', 'Saint Kitts and Nevis', 'Central America'), 
    ('KY', 'Cayman Islands', 'Central America'), 
    ('AI', 'Anguilla', 'Central America'), 
    ('GD', 'Grenada', 'Central America'), 
    ('LC', 'Saint Lucia', 'Central America'), 
    ('VC', 'Saint Vincent and the Grenadines', 'Central America'), 
    ('TC', 'Turks and Caicos Islands', 'Central America'), 
    ('BB', 'Barbados', 'Central America'), 
    ('AG', 'Antigua and Barbuda', 'Central America'), 
    ('SX', 'Sint Maarten (Dutch Part)', 'Central America'), 
    ('DM', 'Dominica', 'Central America'), 
    ('TT', 'Trinidad and Tobago', 'Central America'), 
    ('BS', 'Bahamas', 'Central America'), 
    ('BZ', 'Belize', 'Central America'), 
    ('US', 'United States', 'North America'), 
    ('CA', 'Canada', 'North America'), 
    ('BR', 'Brazil', 'South America'), 
    ('CL', 'Chile', 'South America'), 
    ('AR', 'Argentina', 'South America'), 
    ('SR', 'Suriname', 'South America'), 
    ('GY', 'Guyana', 'South America'), 
    ('VE', 'Venezuela, Bolivarian Republic of', 'South America'), 
    ('UY', 'Uruguay', 'South America'), 
    ('PY', 'Paraguay', 'South America'), 
    ('EC', 'Ecuador', 'South America'), 
    ('CO', 'Colombia', 'South America'), 
    ('PE', 'Peru', 'South America'), 
    ('BO', 'Bolivia, Plurinational State of', 'South America'), 
    ('AW', 'Aruba', 'South America'), 
    ('GS', 'South Georgia and the South Sandwich Islands', 'South America'), 
    ('FK', 'Falkland Islands (Malvinas)', 'South America'), 
    ('CW', 'Curacao', 'South America'), 
    ('TD', 'Chad', 'Africa'), 
    ('DZ', 'Algeria', 'Africa'), 
    ('EG', 'Egypt', 'Africa'), 
    ('LY', 'Libya', 'Africa'), 
    ('MA', 'Morocco', 'Africa'), 
    ('EH', 'Western Sahara', 'Africa'), 
    ('SD', 'Sudan', 'Africa'), 
    ('TN', 'Tunisia', 'Africa'), 
    ('NE', 'Niger', 'Africa'), 
    ('MR', 'Mauritania', 'Africa'), 
    ('ML', 'Mali', 'Africa'), 
    ('BF', 'Burkina Faso', 'Africa'), 
    ('ER', 'Eritrea', 'Africa'), 
    ('SN', 'Senegal', 'Africa'), 
    ('GM', 'Gambia', 'Africa'), 
    ('GW', 'Guinea-Bissau', 'Africa'), 
    ('GN', 'Guinea', 'Africa'), 
    ('SL', 'Sierra Leone', 'Africa'), 
    ('LR', 'Liberia', 'Africa'), 
    ('CI', 'Cote d''Ivoire', 'Africa'),
    ('GH', 'Ghana', 'Africa'),
    ('TG', 'Togo', 'Africa'),
    ('BJ', 'Benin', 'Africa'),
    ('CM', 'Cameroon', 'Africa'),
    ('CF', 'Central African Republic', 'Africa'),
    ('SS', 'South Sudan', 'Africa'),
    ('ET', 'Ethiopia', 'Africa'),
    ('DJ', 'Djibouti', 'Africa'),
    ('SO', 'Somalia', 'Africa'),
    ('GQ', 'Equatorial Guinea', 'Africa'),
    ('GA', 'Gabon', 'Africa'),
    ('KE', 'Kenya', 'Africa'),
    ('TZ', 'Tanzania, United Republic of', 'Africa'),
    ('UG', 'Uganda', 'Africa'),
    ('BI', 'Burundi', 'Africa'),
    ('RW', 'Rwanda', 'Africa'),
    ('CD', 'Congo, the Democratic Republic of the', 'Africa'),
    ('CG', 'Congo', 'Africa'),
    ('AO', 'Angola', 'Africa'),
    ('ZM', 'Zambia', 'Africa'),
    ('MZ', 'Mozambique', 'Africa'),
    ('MW', 'Malawi', 'Africa'),
    ('ZW', 'Zimbabwe', 'Africa'),
    ('NA', 'Namibia', 'Africa'),
    ('BW', 'Botswana', 'Africa'),
    ('SZ', 'Swaziland', 'Africa'),
    ('LS', 'Lesotho', 'Africa'),
    ('MG', 'Madagascar', 'Africa'),
    ('NG', 'Nigeria', 'Africa'),
    ('ZA', 'South Africa', 'Africa'),
    ('SH', 'Saint Helena, Ascension and Tristan Da Cunha', 'Africa'),
    ('SC', 'Seychelles', 'Africa'),
    ('CV', 'Cape Verde', 'Africa'),
    ('ST', 'Sao Tome and Principe', 'Africa'),
    ('MU', 'Mauritius', 'Africa'),
    ('KM', 'Comoros', 'Africa'),
    ('FR', 'France', 'Europe'),
    ('SI', 'Slovenia', 'Europe'),
    ('XK', 'Kosovo', 'Europe'),
    ('RS', 'Serbia', 'Europe'),
    ('ME', 'Montenegro', 'Europe'),
    ('MK', 'Macedonia, the Former Yugoslav Republic of', 'Europe'),
    ('GR', 'Greece', 'Europe'),
    ('HR', 'Croatia', 'Europe'),
    ('BA', 'Bosnia and Herzegovina', 'Europe'),
    ('AL', 'Albania', 'Europe'),
    ('VA', 'Holy See (Vatican City State)', 'Europe'),
    ('SM', 'San Marino', 'Europe'),
    ('IT', 'Italy', 'Europe'),
    ('SK', 'Slovakia', 'Europe'),
    ('RO', 'Romania', 'Europe'),
    ('PL', 'Poland', 'Europe'),
    ('MD', 'Moldova, Republic of', 'Europe'),
    ('HU', 'Hungary', 'Europe'),
    ('CZ', 'Czech Republic', 'Europe'),
    ('BG', 'Bulgaria', 'Europe'),
    ('AT', 'Austria', 'Europe'),
    ('CH', 'Switzerland', 'Europe'),
    ('DE', 'Germany', 'Europe'),
    ('DK', 'Denmark', 'Europe'),
    ('NO', 'Norway', 'Europe'),
    ('SE', 'Sweden', 'Europe'),
    ('FI', 'Finland', 'Europe'),
    ('EE', 'Estonia', 'Europe'),
    ('LV', 'Latvia', 'Europe'),
    ('LT', 'Lithuania', 'Europe'),
    ('BY', 'Belarus', 'Europe'),
    ('NL', 'Netherlands', 'Europe'),
    ('LU', 'Luxembourg', 'Europe'),
    ('BE', 'Belgium', 'Europe'),
    ('GB', 'United Kingdom', 'Europe'),
    ('IE', 'Ireland', 'Europe'),
    ('IS', 'Iceland', 'Europe'),
    ('AD', 'Andorra', 'Europe'),
    ('ES', 'Spain', 'Europe'),
    ('PT', 'Portugal', 'Europe'),
    ('CY', 'Cyprus', 'Europe'),
    ('TR', 'Turkey', 'Europe'),
    ('UA', 'Ukraine', 'Europe'),
    ('IM', 'Isle of Man', 'Europe'),
    ('MC', 'Monaco', 'Europe'),
    ('GI', 'Gibraltar', 'Europe'),
    ('GG', 'Guernsey', 'Europe'),
    ('JE', 'Jersey', 'Europe'),
    ('LI', 'Liechtenstein', 'Europe'),
    ('MT', 'Malta', 'Europe'),
    ('FO', 'Faroe Islands', 'Europe'),
    ('LK', 'Sri Lanka', 'Asia'),
    ('TW', 'Taiwan', 'Asia'),
    ('VN', 'Viet Nam', 'Asia'),
    ('MM', 'Myanmar', 'Asia'),
    ('KH', 'Cambodia', 'Asia'),
    ('LA', 'Lao People''s Democratic Republic', 'Asia'),
    ('TH', 'Thailand', 'Asia'), 
    ('PH', 'Philippines', 'Asia'), 
    ('MY', 'Malaysia', 'Asia'), 
    ('OM', 'Oman', 'Asia'), 
    ('AE', 'United Arab Emirates', 'Asia'), 
    ('YE', 'Yemen', 'Asia'), 
    ('QA', 'Qatar', 'Asia'), 
    ('KW', 'Kuwait', 'Asia'), 
    ('SA', 'Saudi Arabia', 'Asia'), 
    ('IL', 'Israel', 'Asia'), 
    ('LB', 'Lebanon', 'Asia'), 
    ('SY', 'Syrian Arab Republic', 'Asia'), 
    ('JO', 'Jordan', 'Asia'), 
    ('IQ', 'Iraq', 'Asia'), 
    ('CU', 'Cuba', 'Central America'),
    ('RU', 'Russian Federation', 'Asia'), 
    ('AZ', 'Azerbaijan', 'Asia'), 
    ('AM', 'Armenia', 'Asia'), 
    ('GE', 'Georgia', 'Asia'), 
    ('KP', 'Korea, Democratic People''s Republic of', 'Asia'),
    ('KR', 'Korea, Republic of', 'Asia'),
    ('JP', 'Japan', 'Asia'),
    ('BD', 'Bangladesh', 'Asia'),
    ('BT', 'Bhutan', 'Asia'),
    ('NP', 'Nepal', 'Asia'),
    ('MN', 'Mongolia', 'Asia'),
    ('AF', 'Afghanistan', 'Asia'),
    ('PK', 'Pakistan', 'Asia'),
    ('KG', 'Kyrgyzstan', 'Asia'),
    ('IR', 'Iran, Islamic Republic of', 'Asia'),
    ('TM', 'Turkmenistan', 'Asia'),
    ('TJ', 'Tajikistan', 'Asia'),
    ('UZ', 'Uzbekistan', 'Asia'),
    ('IN', 'India', 'Asia'),
    ('KZ', 'Kazakhstan', 'Asia'),
    ('CN', 'China', 'Asia'),
    ('HK', 'Hong Kong', 'Asia'),
    ('SG', 'Singapore', 'Asia'),
    ('MV', 'Maldives', 'Asia'),
    ('BH', 'Bahrain', 'Asia'),
    ('PS', 'Palestine, State of', 'Asia'),
    ('KI', 'Kiribati', 'Australia and Oceania'),
    ('NZ', 'New Zealand', 'Australia and Oceania'),
    ('BN', 'Brunei Darussalam', 'Australia and Oceania'),
    ('TL', 'Timor-Leste', 'Australia and Oceania'),
    ('ID', 'Indonesia', 'Australia and Oceania'),
    ('PG', 'Papua New Guinea', 'Australia and Oceania'),
    ('AU', 'Australia', 'Australia and Oceania'),
    ('TK', 'Tokelau', 'Australia and Oceania'),
    ('NF', 'Norfolk Island', 'Australia and Oceania'),
    ('GU', 'Guam', 'Australia and Oceania'),
    ('PN', 'Pitcairn', 'Australia and Oceania'),
    ('NR', 'Nauru', 'Australia and Oceania'),
    ('TV', 'Tuvalu', 'Australia and Oceania'),
    ('MH', 'Marshall Islands', 'Australia and Oceania'),
    ('AS', 'American Samoa', 'Australia and Oceania'),
    ('CK', 'Cook Islands', 'Australia and Oceania'),
    ('NU', 'Niue', 'Australia and Oceania'),
    ('TO', 'Tonga', 'Australia and Oceania'),
    ('PW', 'Palau', 'Australia and Oceania'),
    ('MP', 'Northern Mariana Islands', 'Australia and Oceania'),
    ('FM', 'Micronesia, Federated States of', 'Australia and Oceania'),
    ('WS', 'Samoa', 'Australia and Oceania'),
    ('VU', 'Vanuatu', 'Australia and Oceania'),
    ('FJ', 'Fiji', 'Australia and Oceania'),
    ('SB', 'Solomon Islands', 'Australia and Oceania')
ON CONFLICT (iso_country_code) DO UPDATE SET nation = EXCLUDED.nation, continent = EXCLUDED.continent;

DELETE FROM nationalities WHERE continent IS NULL;

ALTER TABLE nationalities ALTER COLUMN continent SET NOT NULL;

INSERT INTO subdivisions (iso_code, name, nation) 
VALUES
    ('WA', 'Washington', 'US'),
    ('MD', 'Maryland', 'US'), 
    ('WV', 'West Virginia', 'US'), 
    ('NY', 'New York', 'US'), 
    ('NJ', 'New Jersey', 'US'), 
    ('PA', 'Pennsylvania', 'US'), 
    ('VA', 'Virginia', 'US'), 
    ('KY', 'Kentucky', 'US'), 
    ('OH', 'Ohio', 'US'), 
    ('IN', 'Indiana', 'US'), 
    ('IL', 'Illinois', 'US'), 
    ('MI', 'Michigan', 'US'), 
    ('WI', 'Wisconsin', 'US'), 
    ('CT', 'Connecticut', 'US'), 
    ('RI', 'Rhode Island', 'US'), 
    ('VT', 'Vermont', 'US'), 
    ('NH', 'New Hampshire', 'US'),
    ('MA', 'Massachusetts', 'US'), 
    ('ME', 'Maine', 'US'), 
    ('AL', 'Alabama', 'US'), 
    ('GA', 'Georgia', 'US'), 
    ('SC', 'South Carolina', 'US'), 
    ('FL', 'Florida', 'US'), 
    ('MS', 'Mississippi', 'US'), 
    ('TN', 'Tennessee', 'US'), 
    ('NC', 'North Carolina', 'US'), 
    ('TX', 'Texas', 'US'), 
    ('OK', 'Oklahoma', 'US'),
    ('NM', 'New Mexico', 'US'),
    ('NE', 'Nebraska', 'US'), 
    ('SD', 'South Dakota', 'US'), 
    ('KS', 'Kansas', 'US'), 
    ('CO', 'Colorado', 'US'), 
    ('ND', 'North Dakota', 'US'), 
    ('AR', 'Arkansas', 'US'), 
    ('MO', 'Missouri', 'US'), 
    ('LA', 'Louisiana', 'US'),
    ('IA', 'Iowa', 'US'), 
    ('MN', 'Minnesota', 'US'), 
    ('AZ', 'Arizona', 'US'), 
    ('NV', 'Nevada', 'US'), 
    ('CA', 'California', 'US'), 
    ('UT', 'Utah', 'US'), 
    ('OR', 'Oregon', 'US'), 
    ('MT', 'Montana', 'US'), 
    ('ID', 'Idaho', 'US'), 
    ('WY', 'Wyoming', 'US'), 
    ('HI', 'Hawaii', 'US'), 
    ('AK', 'Alaska', 'US'), 
    ('DC', 'Washington, District of Columbia', 'US'),
    ('DE', 'Delaware', 'US'),
    ('MB', 'Manitoba', 'CA'), 
    ('NT', 'Northwest Territories', 'CA'), 
    ('NL', 'Newfoundland and Labrador', 'CA'), 
    ('NU', 'Nunavut', 'CA'), 
    ('QC', 'Quebec', 'CA'),
    ('BC', 'British Columbia', 'CA'), 
    ('SK', 'Saskatchewan', 'CA'), 
    ('AB', 'Alberta', 'CA'), 
    ('ON', 'Ontario', 'CA'), 
    ('NB', 'New Brunswick', 'CA'), 
    ('NS', 'Nova Scotia', 'CA'), 
    ('PE', 'Prince Edward Island', 'CA'), 
    ('YT', 'Yukon', 'CA'), 
    ('SCT', 'Scotland', 'GB'), 
    ('WLS', 'Wales', 'GB'), 
    ('ENG', 'England', 'GB'),
    ('NIR', 'Northern Ireland', 'GB'), 
    ('ACT', 'Australian Capital Territory', 'AU'), 
    ('TAS', 'Tasmania', 'AU'),
    ('NT', 'Northern Territory', 'AU'), 
    ('WA', 'Western Australia', 'AU'), 
    ('QLD', 'Queensland', 'AU'), 
    ('NSW', 'New South Wales', 'AU'),
    ('VIC', 'Victoria', 'AU'), 
    ('SA', 'South Australia', 'AU');

ALTER TABLE players ADD COLUMN subdivision VARCHAR(3) DEFAULT NULL;

ALTER TABLE player_modifications ADD COLUMN nationality VARCHAR(2) DEFAULT NULL,
                                 ADD COLUMN subdivision VARCHAR(3) DEFAULT NULL;

CREATE OR REPLACE FUNCTION audit_player_modification() RETURNS trigger as $player_modification_trigger$
DECLARE
    name_change CITEXT;
    banned_change BOOLEAN;
    nationality_change VARCHAR(2);
    subdivision_change VARCHAR(3);
BEGIN
    IF (OLD.name <> NEW.name) THEN
        name_change = OLD.name;
    END IF;

    IF (OLD.banned <> NEW.banned) THEN
        banned_change = OLD.banned;
    END IF;

    IF (OLD.nationality <> NEW.nationality) THEN
        nationality_change = OLD.nationality;
    end if;

    IF (OLD.subdivision <> NEW.subdivision) THEN
        subdivision_change = OLD.subdivision;
    end if;

    INSERT INTO player_modifications (userid, id, name, banned, nationality, subdivision)
        (SELECT id, NEW.id, name_change, banned_change, nationality_change, subdivision_change FROM active_user LIMIT 1);

    RETURN NEW;
END;
$player_modification_trigger$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION audit_player_deletion() RETURNS trigger AS $player_deletion_trigger$
BEGIN
    INSERT INTO player_modifications (userid, id, name, banned, nationality, subdivision)
        (SELECT id, OLD.id, OLD.name, OLD.banned, OLD.nationality, OLD.subdivision
         FROM active_user LIMIT 1);

    INSERT INTO player_deletions (userid, id)
        (SELECT id, OLD.id FROM active_user LIMIT 1);

    RETURN NULL;
END;
$player_deletion_trigger$ LANGUAGE plpgsql;

CREATE OR REPLACE VIEW players_with_score AS
SELECT players.id,
       players.name,
       RANK() OVER(ORDER BY scores.total_score DESC) AS rank,
       CASE WHEN scores.total_score IS NULL THEN 0.0::FLOAT ELSE scores.total_score END AS score,
       ROW_NUMBER() OVER(ORDER BY scores.total_score DESC) AS index,
       nationalities.iso_country_code,
       nationalities.nation,
       players.subdivision,
       nationalities.continent
FROM
    (
        SELECT pseudo_records.player,
               SUM(record_score(pseudo_records.progress::FLOAT, pseudo_records.position::FLOAT, 100::FLOAT, pseudo_records.requirement)) as total_score
        FROM (
                 SELECT player,
                        progress,
                        position,
                        CASE WHEN demons.position > 75 THEN 100 ELSE requirement END AS requirement
                 FROM records
                          INNER JOIN demons
                                     ON demons.id = demon
                 WHERE demons.position <= 150 AND status_ = 'APPROVED' AND (demons.position <= 75 OR progress = 100)

                 UNION

                 SELECT verifier as player,
                        CASE WHEN demons.position > 150 THEN 0.0::FLOAT ELSE 100.0::FLOAT END as progress,
                        position,
                        100.0::FLOAT
                 FROM demons

                 UNION

                 SELECT publisher as player,
                        0.0::FLOAT as progress,
                        position,
                        100.0::FLOAT
                 FROM demons

                 UNION

                 SELECT creator as player,
                        0.0::FLOAT as progress,
                        1.0::FLOAT as position, -- doesn't matter
                        100.0::FLOAT
                 FROM creators
             ) AS pseudo_records
        GROUP BY player
    ) scores
        INNER JOIN players
                   ON scores.player = players.id
        LEFT OUTER JOIN nationalities
                        ON players.nationality = nationalities.iso_country_code
WHERE NOT players.banned AND players.id != 1534;
