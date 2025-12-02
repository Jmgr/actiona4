# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_actiona_ng_cli_global_optspecs
	string join \n debug disable-updates= disable-telemetry display= h/help
end

function __fish_actiona_ng_cli_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_actiona_ng_cli_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_actiona_ng_cli_using_subcommand
	set -l cmd (__fish_actiona_ng_cli_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_needs_command" -l disable-updates -d 'Should Actiona-ng check for updates once per day? Default is true' -r -f -a "true\t''
false\t''"
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_needs_command" -l display -d 'X11 display to use (Linux/X11 only 🐧)' -r
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_needs_command" -l debug -d 'Show debug information'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_needs_command" -l disable-telemetry -d 'Should Actiona-ng send anonymous telemetry data? Default is false'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_needs_command" -s h -l help -d 'Print help'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_needs_command" -f -a "run" -d '🤖 runs a script'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_needs_command" -f -a "eval" -d '🧪 evaluates code'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_needs_command" -f -a "repl" -d '💻 starts the interactive terminal (REPL)'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_using_subcommand run" -s h -l help -d 'Print help'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_using_subcommand eval" -s h -l help -d 'Print help'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_using_subcommand repl" -s h -l help -d 'Print help'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_using_subcommand help; and not __fish_seen_subcommand_from run eval repl help" -f -a "run" -d '🤖 runs a script'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_using_subcommand help; and not __fish_seen_subcommand_from run eval repl help" -f -a "eval" -d '🧪 evaluates code'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_using_subcommand help; and not __fish_seen_subcommand_from run eval repl help" -f -a "repl" -d '💻 starts the interactive terminal (REPL)'
complete -c actiona-ng-cli -n "__fish_actiona_ng_cli_using_subcommand help; and not __fish_seen_subcommand_from run eval repl help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
