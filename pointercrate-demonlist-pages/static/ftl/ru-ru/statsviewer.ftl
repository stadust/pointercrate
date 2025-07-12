statsviewer = Панель статистики
    .rank = Позиция в демонлисте
    .score = Очки демонлиста
    .stats = Статистика в демонлисте
    .hardest = Сложнейший демон

    .completed = Пройденные демоны
    .completed-main = Пройденные демоны из Main-листа
    .completed-extended = Пройденные демоны из Extended-листа
    .completed-legacy = Пройденные демоны из Legacy-листа

    .created = Созданные демоны
    .published = Опубликованные демоны
    .verified = Верифнутые демоны
    .progress = Прогрессы

    .stats-value = { $main } Main, { $extended } Extended, { $legacy } Legacy
    .value-none = Н/Д

statsviewer-individual = Игроки
    .welcome = Нажмите на имя игрока слева для начала работы!

    .option-international = Международный

statsviewer-nation = Страны
    .welcome = Нажмите на имя страны слева для начала работы!

    .players = Игроки
    .unbeaten = Непройденные демоны

    .created-tooltip = Был создан { $players } { $players ->
            [one] игрок
            [few] игрока
            [many] игроков
            *[other] игроков
        } в этой стране:
    .published-tooltip = Был опубликован:
    .verified-tooltip = Был верифицирован:
    .beaten-tooltip = Был пройден { $players } { $players ->
            [one] игрок
            [few] игрока
            [many] игроков
            *[other] игроков
        } в этой стране:
    .progress-tooltip = Был достигнут { $players } { $players ->
            [one] игрок
            [few] игрока
            [many] игроков
            *[other] игроков
        } в этой стране:

demon-sorting-panel = Сортировка демонов
    .info = Порядок, в котором пройденные демоны должны отображаться

    .option-alphabetical = По алфавиту
    .option-position = По позиции

continent-panel = Континент
    .info = Выберите континент ниже, чтобы сфокусировать панель статистики на данном континенте. Выберите 'Все' для сброса фильтра.

    .option-all = Все

    .option-asia = Азия
    .option-europe = Европа
    .option-australia = Австралия
    .option-africa = Африка
    .option-northamerica = Северная Америка
    .option-southamerica = Южная Америка
    .option-centralamerica = Центральная Америка

toggle-subdivision-panel = Показать подразделения
    .info = Настройка отображения на карте политических подразделений.

    .option-toggle = Показать политические подразделения

# { $countries } will be replaced with .info-countries, which will be
# turned into a tooltip listing all of the selectable countries
subdivision-panel = Политическое подразделение
    .info = Для { $countries } вы можете выбрать штат либо регион из выпадающего списка ниже, чтобы сфокусировать панель статистики на выбранном штате либо регионе.
    .info-countries = следующих стран

    .option-none = Н/Д
