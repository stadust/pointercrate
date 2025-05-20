statsviewer = Stats Viewer
    .rank = Demonlist rank
    .score = Demonlist score
    .stats = Demonlist stats
    .hardest = Hardest demon

    .completed = Demons completed
    .completed-main = Main list demons completed
    .completed-extended = Extended list demons completed
    .completed-legacy = Legacy list demons completed

    .created = Demons created
    .published = Demons published
    .verified = Demons verified
    .progress = Progress on

    .stats-value = { $main } Main, { $extended } Extended, { $legacy } Legacy
    .value-none = None

statsviewer-individual = Individual
    .welcome = Click on a player's name on the left to get started!

    .option-international = International

statsviewer-nation = Nations
    .welcome = Click on a country's name on the left to get started!

    .players = Players
    .unbeaten = Unbeaten demons

    .created-tooltip = (Co)created by { $players } { $players ->
            [one] player
            *[other] players
        } in this country:
    .published-tooltip = Published by:
    .verified-tooltip = Verified by:
    .beaten-tooltip = Beaten by { $players } { $players ->
            [one] player
            *[other] players
        } in this country:
    .progress-tooltip = Achieved by { $players } { $players ->
            [one] player
            *[other] players
        } in this country:

demon-sorting-panel = Demon Sorting
    .info = The order in which completed demons should be listed

    .option-alphabetical = Alphabetical
    .option-position = Position

continent-panel = Continent
    .info = Select a continent below to focus the stats viewer to that continent. Select 'All' to reset selection.

    .option-all = All

    .option-asia = Asia
    .option-europe = Europe
    .option-australia = Australia
    .option-africa = Africa
    .option-northamerica = North America
    .option-southamerica = South America
    .option-centralamerica = Central America

toggle-subdivision-panel = Show Subdivisions
    .info = Whether the map should display political subdivisions.

    .option-toggle = Show political subdivisions

# { $countries } will be replaced with .info-countries, which will be
# turned into a tooltip listing all of the selectable countries
subdivision-panel = Political Subdivision
    .info = For the { $countries } you can select a state/province from the dropdown below to focus the stats viewer to that state/province.
    .info-countries = following countries

    .option-none = None
