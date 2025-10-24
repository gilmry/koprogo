Feature: Internationalization translations
  As a user
  I want localized messages
  So that I see errors in my language

  Scenario Outline: Translate keys to target language
    When I translate key "<key>" to "<lang>"
    Then the translation should be "<text>"

    Examples:
      | key                     | lang | text                                            |
      | BuildingNameEmpty       | fr   | Le nom du bâtiment ne peut pas être vide        |
      | TotalUnitsMustBePositive| nl   | Totaal aantal eenheden moet groter zijn dan 0   |
      | NotFound                | en   | Not found                                       |

