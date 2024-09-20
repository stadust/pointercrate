-- Your SQL goes here

CREATE OR REPLACE FUNCTION record_score(progress FLOAT, demon FLOAT, list_size FLOAT, requirement FLOAT) RETURNS FLOAT AS
$record_score$
SELECT CASE
           WHEN progress = 100 THEN
                   CASE
                       
                       WHEN 125 < demon AND demon <= 150 THEN
                            150.0 * EXP(((1.0 - demon) * LN(1.0 / 30.0)) / -149.0)
                       WHEN 50 < demon AND demon <= 125 THEN
                            60 * (EXP(LN(2.333) * ((51.0 - demon) * (LN(30.0) / 99.0)))) + 1.884
                       WHEN 20 < demon AND demon <= 50 THEN
                            -100.0 * (EXP(LN(1.01327) * (demon - 26.489))) + 200.0
                       WHEN demon <= 20 THEN
                            (250 - 100.39) * (EXP(LN(1.168) * (1 - demon))) + 100.39
                   
                   END
                                                                 
           WHEN progress < requirement THEN
               0.0
           ELSE
                       CASE
                       
                       WHEN 125 < demon AND demon <= 150 THEN
                            150.0 * EXP(((1.0 - demon) * LN(1.0 / 30.0)) / -149.0) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10
                       WHEN 50 < demon AND demon <= 125 THEN
                            (60 * (EXP(LN(2.333) * ((51.0 - demon) * (LN(30.0) / 99.0)))) + 1.884) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10
                       WHEN 20 < demon AND demon <= 50 THEN
                            (-100.0 * (EXP(LN(1.01327) * (demon - 26.489))) + 200.0) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10
                       WHEN demon <= 20 THEN
                            ((250 - 100.39) * (EXP(LN(1.168) * (1 - demon))) + 100.39) * (EXP(LN(5) * (progress - requirement) / (100 - requirement))) / 10
                   
                       END
           END;
$record_score$
    LANGUAGE SQL IMMUTABLE;

