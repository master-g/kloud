## ADDED Requirements

### Requirement: Search mode activation
Pressing `/` when not in input mode SHALL activate search mode. A search bar SHALL appear at the top of the message area. Typing SHALL filter and highlight matching text in real-time.

#### Scenario: Activate search
- **WHEN** user presses `/` outside of input mode
- **THEN** a search bar SHALL appear at the top of the message area
- **THEN** the cursor SHALL be placed in the search input

#### Scenario: Exit search
- **WHEN** user presses `Esc` during search
- **THEN** the search bar SHALL disappear
- **THEN** all highlight markers SHALL be cleared
- **THEN** the view SHALL return to the previous scroll position

### Requirement: Substring matching and highlighting
Search SHALL perform case-insensitive substring matching across all visible message text. Matches SHALL be highlighted with the `suggestion` color background.

#### Scenario: Multiple matches
- **WHEN** search term "error" matches 5 locations across messages
- **THEN** all 5 locations SHALL be highlighted
- **THEN** the current match SHALL have a brighter highlight

### Requirement: Navigate between matches
In search mode, `n` SHALL jump to the next match, `N` SHALL jump to the previous match. The view SHALL scroll to ensure the current match is visible.

#### Scenario: Navigate forward through matches
- **WHEN** user presses `n` in search mode
- **THEN** the highlight SHALL move to the next match
- **THEN** the message area SHALL scroll to center the match vertically

#### Scenario: Wrap around
- **WHEN** the last match is current and user presses `n`
- **THEN** the search SHALL wrap to the first match
- **THEN** a brief "search wrapped" indicator SHALL appear

### Requirement: Match count display
The search bar SHALL display the current match index and total count (e.g., `[3/12]`).

#### Scenario: Display match count
- **WHEN** search finds 12 matches
- **THEN** the search bar SHALL show `[1/12]` initially
- **WHEN** user navigates to the 3rd match
- **THEN** the display SHALL update to `[3/12]`
