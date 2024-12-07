## Adding a new display option

1. Add a field to `Cli`

2. Add a field to `DisplayFields`

3. Add a formatter to `text` and a renderer to `pretty`

4. Follow [[Adding a new interactive command]]

## Adding a new interactive command

1. Add new help menu
  - in `render_interactive_help` always
  - in `render_menu` also if it affects the main display
  
2. Add new match arm in `run` (or relevant `event::poll` match)

3. Add new entry in `DisplayFields` and `Cli`