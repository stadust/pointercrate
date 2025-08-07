user-username = Логин:
user-displayname = Отображаемый ник:
    .none = Н/Д
user-id = ID пользователя:

user-permissions = Полномочия:
    .moderator = Модератор
    .administrator = Администратор

    .list-helper = Помощник листа
    .list-moderator = Модератор листа
    .list-administrator = Лидер листа

## Auth input fields
auth-username = Логин:
    .validator-valuemissing = Требуется логин
    .validator-tooshort = Логин слишком короткий. Он должен быть как минимум 3 символа в длину.
    .error-alreadytaken = Этот логин уже занят. Пожалуйста, выберите другой
auth-password = Пароль:
    .validator-valuemissing = Требуется пароль
    .validator-tooshort = Пароль слишком короткий. Он должен быть как минимум 10 символов в длину.
auth-repeatpassword = Повторите пароль:
    .validator-notmatching = Пароли не совпадают

## Login/registration forms
#
# The .redirect-link attributes will be turned into
# clickable link, which will replace { $redirect-link }
# in the .redirect attributes
#
login = Вход
    .oauth-info = Если вы связали свой аккаунт pointercrate с аккаунтом Google, вы должны войти через Google, нажав на кнопку ниже:

    .methods-separator = либо

    .info = Войдите в аккаунт через свой логин и пароль. Попытки входа ограничены в качестве 3 попыток раз в 30 минут.
    .submit = Войти

    .error-invalidcredentials = Неверные данные

    .redirect = Уже есть аккаунт pointercrate? { $redirect-link } в него.
    .redirect-link = Войдите

register = Регистрация
    .info = Здесь проходит создание нового аккаунта. Учтите, что логин нельзя поменять после создания аккаунта, поэтому выбирайте его с умом!
    .submit = Зарегистрироваться

    .redirect = Еще нет аккаунта pointercrate? { $redirect-link } его!
    .redirect-link = Зарегистрируйте

register-oauth = Выберите ник:
    .submit = Зарегистрироваться!

## Users tab
users = Пользователи

user-viewer = Менеджер аккаунтов pointercrate
    .welcome = Нажмите на пользователя слева для начала работы!
    .delete-user = Удалить пользователя
    .edit-user = Изменить пользователя

    .edit-success = Пользователь успешно изменен!
    .edit-notmodified = Изменений не было сделано!
    .delete-success = Пользователь успешно удален!

    .own-account = Это ваш аккаунт. Вы не можете менять свой аккаунт через этот интерфейс!

user-listed = ID: { $user-id }
    .displayname = Отображаемое имя:

user-idsearch-panel = Найти пользователей
    .info = Пользователей можно опознать по имени и ID. Для изменения аккаунта пользователя вам нужен их ID. Если вам ничего неизвестно о нем, попробуйте найти его в списке ниже
    .id-field = ID пользователя:

    .submit = Найти по ID

    .id-validator-valuemissing = Требуется ID пользователя

## Profile tab
profile = Профиль
    .header = Профиль - {$username}

profile-username = Логин
    .info = Имя, под которым вы зарегистрировались, и которые вы используете для входа в pointercrate. Это имя уникально для вашего аккаунта и не может быть изменено.

profile-display-name = Отображаемое имя
    .info = При установлении это имя будет отображаться в панели с именами команды листа вместо вашего логина. Отображаемые имена не уникальны, и вы не можете использовать их для входа в аккаунт pointercrate.

    .dialog-header = Изменение отображаемого имени
    .dialog-newname = Новое отображаемое имя:

    .dialog-submit = Изменить

profile-youtube = Канал YouTube
    .info = Ссылка на ваш YouTube-канал, если он у вас есть. При его установлении все упоминания вашего имени превратятся в гиперссылку с переходом на этот канал.

    .dialog-header = Изменение ссылки на YouTube-канал
    .dialog-newlink = Новая ссылка на канал:

    .dialog-submit = Изменить

    .newlink-validator-typemismatch = Пожалуйста, введите правильную ссылку

profile-permissions = Полномочия
    .info = Полномочия, которые вы имеете в pointercrate. '... листа' означает, что вы член команды демонлиста. 'Модератор' и 'Администратор' причисляют вас к стаффу pointercrate.

profile-delete-account = Удалить мой аккаунт
    .dialog-header = Удаление аккаунта
    .dialog-info = Удаление вашего аккаунта невозвратимо!
    .dialog-submit = Удалить

profile-change-password = Поменять пароль
    .dialog-header = Изменение пароля
    .dialog-info = Для всех связанных с профилем изменений нужно повторно ввести ваш пароль. Изменение пароля выкинет вас из аккаунта и осуществит переход на страницу входа. Это также отключит все токены доступа к вашему аккаунту.

    .dialog-newpassword = Новый пароль:
    .dialog-repeatnewpassword = Повторите новый пароль:
    .dialog-authenticate = Аутентификация:

    .dialog-submit = Изменить

    .authenticate-validator-valuemissing = Требуется пароль
    .authenticate-validator-tooshort = Пароль слишком короткий. Он должен быть как минимум 10 символов в длину.

    .newpassword-validator-tooshort = Пароль слишком короткий. Он должен быть как минимум 10 символов в длину.

    .repeatnewpassword-validator-tooshort = Пароль слишком короткий. Он должен быть как минимум 10 символов в длину.
    .repeatnewpassword-validator-notmatching = Пароли не совпадают

profile-logout = Выход
    .info = Выход из своего аккаунта pointercrate в этом браузере.
    .button = Выйти

profile-get-token = Получение токена доступа
    .info = Ваш токен доступа pointercrate позволяет вам либо авторизованным вами программам делать API-запросы от вашего имени. Через токен доступа при этом нельзя менять данные аккаунта.
    .button = Получить токен доступа

    .view-header = Ваш токен доступа:

profile-invalidate-tokens = Отключить токены
    .info = Если один из ваших токенов доступа был слит в сеть, вы можете отключить их здесь. Отключение приведет к потере функциональности всех токенов доступа, связанных с вашим аккаунтом. Это включает в себя хранящийся в браузере на данный момент токен, что означает необходимость повторной авторизации после выполнения этого действия!
    .button = Отключить все токены доступа

profile-oauth = Связь с Google
    .info = Здесь проходит включение возможности входа в ваш аккаунт pointercrate через Google. Эта опция более защищена, чем традиционный метод входа, и предотвращает потерю доступа к аккаунту из-за забытого пароля. Связь с аккаунтом Google отменить нельзя, и вы не можете поменять связанный аккаунт Google на другой!
