## Demon information, including information fetched by dash-rs
## Fields included in forms may have validators
demon-name = Название демона
    .validator-valuemissing = Пожалуйста, укажите название

demon-password = Пароль от уровня

demon-id = ID уровня
    .validator-rangeunderflow = ID уровня должен быть положительным

demon-length = Длина уровня

demon-objects = Число объектов

demon-difficulty = Внутриигровая сложность

demon-gdversion = Был создан в

demon-ngsong = Песня на Newgrounds

demon-score = Очки демонлиста ({$percent}%)

demon-video = Видео верификации
    .validator-typemismatch = Пожалуйста, укажите правильную ссылку

demon-thumbnail = Превью
    .validator-typemismatch = Пожалуйста, укажите правильную ссылку
    .validator-valuemissing = Пожалуйста, введите ссылку

demon-position = Позиция
    .validator-rangeunderflow = Позиция уровня должна быть равна как минимум 1
    .validator-badinput = Позиция уровня должна быть целым числом
    .validator-stepmismatch = Позиция уровня не должна быть дробной
    .validator-valuemissing = Пожалуйста, укажите позицию

demon-requirement = Требование
    .validator-rangeunderflow = Требование к рекордам не может быть отрицательным
    .validator-rangeoverflow = Требование к рекордам не может быть больше 100%
    .validator-badinput = Требование к рекордам должно быть целым числом
    .validator-stepmismatch = Требование к рекордам не должно быть дробным
    .validator-valuemissing = Пожалуйста, укажите требование к рекордам

demon-publisher = Публикатор
    .validator-valuemissing = Пожалуйста, укажите публикатора

demon-verifier = Верифер
    .validator-valuemissing = Пожалуйста, укажите верифера

demon-creators = Создатели

demon-headline-by = от { $creator }
demon-headline-verified-by = верифицирован { $verifier }
demon-headline-published-by = опубликован { $publisher }

# { $verified-and-published } represents two possible variations of text
# either .same-verifier-publisher OR .unique-verifier-publisher
#
# { $more } in .more-creators is transformed into a tooltip listing all of
# a demon's creators, with the text being .more-creators-tooltip
demon-headline = от { $creator }
    .same-verifier-publisher = верифицирован и опубликован { $publisher }
    .unique-verifier-publisher = { demon-headline-published-by }, { demon-headline-verified-by }

    .no-creators = от Неизвестно, { $verified-and-published }

    .one-creator = { demon-headline-by }, { $verified-and-published }
    .one-creator-is-publisher = { demon-headline-by }, верифицирован { $verifier }
    .one-creator-is-verifier = { demon-headline-by }, опубликован { $publisher }

    .two-creators = от { $creator1 } и { $creator2 }, { $verified-and-published }

    .more-creators = { demon-headline-by } и { $more }, { $verified-and-published }
    .more-creators-tooltip = других

## Position history table
movements = История позиции
    .date = Дата
    .change = Изменение

movements-newposition = Новая позиция
    .legacy = Legacy

movements-reason = Причина
    .added = Добавлен в лист
    .addedabove = { $demon } был добавлен выше
    .moved = Перемещён
    .movedabove = { $demon } был перемещён выше этого демона
    .movedbelow = { $demon } был перемещён ниже этого демона

## Records table
demon-records = Рекорды

demon-records-qualify = {$percent}% { $percent ->
    [100] требуется для квалификации
    *[other] или выше требуется для квалификации
}

demon-records-total = {$num-records} { $num-records ->
    [one] рекорд зарегистрирован
    [few] рекорда зарегистрировано
    [many] рекордов зарегистрировано
    *[other] рекордов зарегистрировано
}, из которых {$num-completions} { $num-completions ->
    [one] рекорд - 100%
    [few] рекорда - 100%
    [many] рекордов - 100%
    *[other] рекордов - 100%
}

## Demons tab in user area
demons = Демоны
demon-manager = Менеджер демонов

demon-listed = {$demon} (ID: {$demon-id})
    .publisher = от {$publisher}

demon-viewer = Демон #
    .welcome = Нажмите на демон слева для начала работы!

    .video-field = { demon-video }:
    .thumbnail-field = { demon-thumbnail }:
    .position-field = { demon-position }:
    .requirement-field = { demon-requirement }:
    .publisher-field = { demon-publisher }:
    .verifier-field = { demon-verifier }:
    .creators-field = { demon-creators }:

demon-add-panel = Добавление демона
    .button = Добавить демон!

# Demon addition form
demon-add-form = Добавление демона
    .name-field = { demon-name }:
    .name-validator-valuemissing = Пожалуйста, укажите название демона

    .levelid-field = ID уровня в Geometry Dash:
    .position-field = { demon-position }:
    .requirement-field = { demon-requirement }:
    .verifier-field = { demon-verifier }:
    .publisher-field = { demon-publisher }:
    .video-field = { demon-video }:
    .creators-field = { demon-creators }:

    .submit = Добавить демон

    .edit-success = Демон добавлен успешно!

# Demon viewer dialogs
demon-video-dialog = Изменение ссылки на видео с верификацией
    .info = Здесь проходит изменение ссылки на видео с верификацией для этого демона. Оставьте ссылку пустой для удаления видео.
    .video-field = Ссылка на видео:
    .submit = Изменить

demon-name-dialog = Изменение названия демона
    .info = Здесь проходит изменение названия данного демона. Возможность добавления нескольких демонов с одинаковыми именами полностью работает!
    .name-field = Название:
    .submit = Изменить

# { $video-id } will be replaced by https://i.ytimg.com/vi/{.info-videoid}/mqdefault.jpg but italicized
# in english, this looks like https://i.ytimg.com/vi/VIDEO_ID/mqdefault.jpg
demon-thumbnail-dialog = Изменение ссылки на превью
    .info = Здесь проходит изменение ссылки на превью видео для этого демона. Чтобы поставить превью конкретного видео на YouTube, измените значение на { $video-id }.
    .info-videoid = VIDEO_ID

    .thumbnail-field = Ссылка на превью:
    .submit = Изменить

demon-position-dialog = Изменение позиции демона
    .info = Здесь проходит изменение позиции данного демона. Позиция должна быть больше 0 и не больше текущего значения размера листа.
    .position-field = Позиция:
    .submit = Изменить

demon-requirement-dialog = Изменение требования к рекордам на демоне
    .info = Здесь проходит изменение требования к рекордам для этого демона. Требование Должно быть между 0 и 100 включительно.
    .requirement-field = Требование:
    .submit = Изменить

demon-publisher-dialog = Изменение публикатора демона
    .info = Здесь проходит введение нового публикатора демона через поле ниже. Если такой игрок уже существует, его имя появится в качестве предложения ниже поля ввода. После этого нажмите на кнопку ниже.
    .submit = Изменить

demon-verifier-dialog = Изменение верифера демона
    .info = Здесь проходит введение нового верифера демона через поле ниже. Если такой игрок уже существует, его имя появится в качестве предложения ниже поля ввода. После этого нажмите на кнопку ниже.
    .submit = Изменить

demon-creator-dialog = Добавление креатора
    .info = Здесь проходит добавление нового креатора для этого демона через поле ниже. Если такой игрок уже существует, его имя появится в качестве предложения ниже поля ввода. После этого нажмите на кнопку ниже.
    .submit = Добавить креатора

    .edit-success = Креатор добавлен успешно!