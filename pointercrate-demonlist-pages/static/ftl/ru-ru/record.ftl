## Commonly referenced record data
record-submitted = Отправлен
record-underconsideration = На рассмотрении
record-approved = Принят
record-rejected = Отклонен

record-videolink = Ссылка на видео
record-videoproof = Видео-доказательства
record-rawfootage = Необработанная запись
record-demon = Демон
record-holder = Владелец рекорда
record-progress = Прогресс
record-submitter = ID отправителя

## Records tab (user area)
records = Рекорды
record-manager = Менеджер рекордов
    .all-option = Все демоны

record-listed = Рекорд #{ $record-id }
    .progress = { $percent }% на { $demon }

record-viewer = Рекорд #
    .welcome = Нажмите на рекорд слева для начала работы!
    .delete = Удалить рекорд

    .copy-data-success = Данные о рекорде скопированы в буфер обмена!
    .copy-data-error = Ошибка копирования в буфер обмена

    .confirm-delete = Вы уверены? Это невозвратно удалит этот рекорд и все заметки на нем!

record-note = Добавить заметку
    .placeholder = Здесь проходит добавление заметок. Нажмите 'Добавить' выше после написания!
    .public-checkbox = Публичная заметка

    .submit = Добавить

record-note-listed = Заметка #{ $note-id }
    .confirm-delete = Это действие невозвратно удалит эту заметку. Продолжить?

    .author = Эта заметка была оставлена { $author }.
    .author-submitter = Эта заметка была оставлена как комментарий от отправителя.
    .editors = Эту заметка позже отредактировали: { $editors }.
    .transferred = Эта заметка изначально не принадлежит этому рекорду.
    .public = Эта заметка является публичной.

record-status-filter-panel = Фильтрация
    .info = Фильтрация по статусу рекордов

record-status-filter-all = Все

record-idsearch-panel = Найти рекорд по ID
    .info = Рекорды можно уникально идентифицировать по их ID. Введение ID рекорда ниже выберет его слева (при условии его существования)
    .id-field = ID рекорда:

    .submit = Найти по ID

    .id-validator-valuemissing = Требуется ID рекорда

record-playersearch-panel = Фильтрация по игроку
    .info = Игроков можно уникально идентифицировать по их имени и ID. Введение любого из них в соответствующем поле ниже отфильтрует список слева. Фильтр сбрасывается через нажатие на "Найти ..." с пустыми полями ввода.

    .id-field = ID игрока:
    .id-submit = Найти по ID

    .name-field = Имя игрока:
    .name-submit = Найти по имени

# Record viewer dialogs
record-videolink-dialog = Изменение ссылки на видео
    .info = Здесь проходит изменение ссылки на видео для данного рекорда. Учтите, что если вы являетесь модератором листа, вам положено оставить это поле пустым для удаления видео с рекорда.
    .videolink-field = Ссылка на видео:

    .submit = Изменить

    .videolink-validator-typemismatch = Пожалуйста, введите правильную ссылку

record-demon-dialog = Изменение демона в рекорде
    .info = Здесь проходит изменение демона, связанного с данным рекордом. Ниже можно найти демон, с которым должен быть этот рекорд. После этого нажмите на него для изменения рекорда

record-holder-dialog = Изменение владельца рекорда
    .info = Здесь проходит изменение владельца данного рекорда через текстовое поле ниже. Если такой игрок уже существует, он появится под полем в качестве предложения. После этого нажмите на кнопку ниже.
    .submit = Изменить

record-progress-dialog = Изменение прогресса в рекорде
    .info = Здесь проходит изменение значения прогресса для данного рекорда. Прогресс должен находиться между изначальным требованием для данного демона и 100 (включительно).
    .progress-field = Прогресс:

    .submit = Изменить

    .progress-validator-rangeunderflow = Значение прогресса не может быть отрицательным
    .progress-validator-rangeoverflow = Значение прогресса не может быть больше 100%
    .progress-validator-badinput = Значение прогресса должно быть целым числом
    .progress-validator-stepmismatch = Значение прогресса не должно быть дробным
    .progress-validator-valuemissing = Пожалуйста, введите значение прогресса

# The giant information box below the record manager, split
# into different sections here
#
# Each section (except .a and .b) will begin with a bolded version of
# the appropriate record state, or a bolded version of .note for .note-a/b
# attributes
#
record-manager-help = Работа с рекордами
    .a = Используйте список слева для выбора рекордов и их последующего просмотра либо изменения. Используйте панель справа для фильтрации списка рекордов по статусу, игроку и т.д. Нажатие на поле { record-status-filter-all } сверху позволяет фильтровать по конкретному демону.

    .b = Рекорд может быть в 4 различных состояниях: { record-rejected }, { record-approved }, { record-submitted } и { record-underconsideration }. Для простоты объяснения представим, что существует игрок по имени Bob, имеющий рекорд на демоне Cataclysm.

    .rejected = Если рекорд { record-rejected }, это означает, что у Bob нет других рекордов на Cataclysm с другими статусами, и Bob далее не может отправить рекорды на Cataclysm. Также это означает, что если у Bob имеется неотклоненный рекорд на Cataclysm, мы можем тут же понять, что у Bob отклоненных рекордов на Cataclysm нет впринципе.
    Отклонение любого рекорда от Bob на Cataclysm удалит все прочие рекорды Bob на Cataclysm для сохранения уникальности, описанной выше.

    .approved = Если рекорд { record-approved }, это означает, что у Bob нет других рекордов с меньшим прогрессом, чем существующий рекорд со статусом { record-approved }, и такие рекорды при этом запрещены для отправки.
    Изменение статуса рекорда на { record-approved } удалит все рекорды от Bob на Cataclysm с меньшим прогрессом.

    .submitted = Если рекорд { record-submitted }, на нем нет никаких ограничений по уникальности. Это означает, что у Bob может быть сразу несколько отправленных рекордов на Cataclysm при условии того, что на каждом из них ссылки на видео разные. Стоит учесть, что по условиям выше все дубликаты удаляются, как только один из рекордов будет { record-approved } либо { record-rejected }.

    .underconsideration = Если рекорд { record-underconsideration }, концептуально он также является отправленным рекордом. Единственная разница заключается в том, что Bob больше не может отправлять рекорды на Cataclysm.

    .note = Заметки

    .note-a = Если игрок забанен, им запрещено иметь рекорды со статусом { record-approved } либо { record-submitted } в листе. Все рекорды, помеченные как '{ record-submitted }' будут удалены, все остальные поменяют статус на '{ record-rejected }'.

    .note-b = Бан отправителя приведет к удалению всех их рекордов со статусом '{ record-submitted }'. Отправленные ими рекорды, которые уже поменяли статус на { record-approved } либо { record-rejected } не будут затронуты.