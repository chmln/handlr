function __handlr_autocomplete
  function subcommands
    set -l handlr_commands 'get help launch list open set unset'
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "get" -d "Show handler for mime"
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "launch" -d "Launch given handler with path/args"
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "list" -d "Show handlers (default applications)"
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "open" -d "Open path/URL with default handler (like xdg-open)"
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "set" -d "Set handler for extension (e.g. pdf) or mime type"
    complete -f -c handlr -n "not __fish_seen_subcommand_from $handlr_commands" -a "unset" -d "Unset handler"
  end

  function _set_add
    complete -f -c handlr -n '__fish_seen_subcommand_from set; __fish_prev_arg_in "set"' -a '(handlr autocomplete -m)'
    complete -f -c handlr -n '__fish_seen_subcommand_from set; set -l last (commandline -pco)[-2]; [ "$last" = "set" ]' -a '(handlr autocomplete -d)'

    complete -f -c handlr -n '__fish_seen_subcommand_from add; __fish_prev_arg_in "add"' -a '(handlr autocomplete -m)'
    complete -f -c handlr -n '__fish_seen_subcommand_from add; set -l last (commandline -pco)[-2]; [ "$last" = "add" ]' -a '(handlr autocomplete -d)'
  end

  subcommands
  _set_add
  complete -f -c handlr -n '__fish_seen_subcommand_from get' -a '(handlr autocomplete -m)'
  complete -f -c handlr -n '__fish_seen_subcommand_from get' -l 'json'
  complete -f -c handlr -n '__fish_seen_subcommand_from unset' -a '(handlr autocomplete -m)'
  complete -f -c handlr -n '__fish_seen_subcommand_from launch; __fish_prev_arg_in launch' -a '(handlr autocomplete -m)'

end

__handlr_autocomplete
