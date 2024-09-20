-- Your SQL goes here

CREATE OR REPLACE FUNCTION record_score(progress FLOAT, demon FLOAT, list_size FLOAT, requirement FLOAT) RETURNS FLOAT AS
$record_score$
SELECT CASE
           WHEN progress = 100 THEN
                   CASE
                       
                       WHEN 55 < demon AND demon <= 150 THEN
                            (56.191 * EXP(LN(2) * ((54.147 - (demon + 3.2)) * LN(50.0)) / 99.0)) + 6.273
                       WHEN 35 < demon AND demon <= 55 THEN
                            212.61 * (EXP(LN(1.036) * (1 - demon))) + 25.071
                       WHEN 20 < demon AND demon <= 35 THEN
                            (250 - 83.389) * (EXP(LN(1.0099685) * (2 - demon))) - 31.152
                       WHEN demon <= 20 THEN
                            (250 - 100.39) * (EXP(LN(1.168) * (1 - demon))) + 100.39
                   
                   END
                                                                 
           WHEN progress < requirement THEN
               0.0
           ELSE
                       CASE
                       
                       WHEN 55 < demon AND demon <= 150 THEN
                            ((56.191 * EXP(LN(2) * ((54.147 - (demon + 3.2)) * LN(50.0)) / 99.0)) + 6.273) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10
                       WHEN 35 < demon AND demon <= 55 THEN
                            (212.61 * (EXP(LN(1.036) * (1 - demon))) + 25.071) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10
                       WHEN 20 < demon AND demon <= 35 THEN
                            ((250 - 83.389) * (EXP(LN(1.0099685) * (2 - demon))) - 31.152) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10
                       WHEN demon <= 20 THEN
                            ((250 - 100.39) * (EXP(LN(1.168) * (1 - demon))) + 100.39) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10
                   
                       END
           END;
$record_score$
    LANGUAGE SQL IMMUTABLE;
