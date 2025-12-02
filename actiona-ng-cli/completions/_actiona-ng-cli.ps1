
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'actiona-ng-cli' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'actiona-ng-cli'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'actiona-ng-cli' {
            [CompletionResult]::new('--disable-updates', '--disable-updates', [CompletionResultType]::ParameterName, 'Should Actiona-ng check for updates once per day? Default is true')
            [CompletionResult]::new('--display', '--display', [CompletionResultType]::ParameterName, 'X11 display to use (Linux/X11 only 🐧)')
            [CompletionResult]::new('--debug', '--debug', [CompletionResultType]::ParameterName, 'Show debug information')
            [CompletionResult]::new('--disable-telemetry', '--disable-telemetry', [CompletionResultType]::ParameterName, 'Should Actiona-ng send anonymous telemetry data? Default is false')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, '🤖 runs a script')
            [CompletionResult]::new('eval', 'eval', [CompletionResultType]::ParameterValue, '🧪 evaluates code')
            [CompletionResult]::new('repl', 'repl', [CompletionResultType]::ParameterValue, '💻 starts the interactive terminal (REPL)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'actiona-ng-cli;run' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'actiona-ng-cli;eval' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'actiona-ng-cli;repl' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'actiona-ng-cli;help' {
            [CompletionResult]::new('run', 'run', [CompletionResultType]::ParameterValue, '🤖 runs a script')
            [CompletionResult]::new('eval', 'eval', [CompletionResultType]::ParameterValue, '🧪 evaluates code')
            [CompletionResult]::new('repl', 'repl', [CompletionResultType]::ParameterValue, '💻 starts the interactive terminal (REPL)')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'actiona-ng-cli;help;run' {
            break
        }
        'actiona-ng-cli;help;eval' {
            break
        }
        'actiona-ng-cli;help;repl' {
            break
        }
        'actiona-ng-cli;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
