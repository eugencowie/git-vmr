#!/usr/bin/env bash
#
# Convert a man page to markdown using pandoc.
#
# Usage: man-to-md.sh <man-page> <path/to/output.md>
#
# Examples:
#   man-to-md.sh git git.md
#   man-to-md.sh git-add git/add.md
#

set -euo pipefail

# Convert the man page to markdown using pandoc
mkdir -p "$(dirname "$2")"
gzip -dc "$(man -w "$1")" | pandoc --from=man --to=gfm --output="$2" -

# Convert the markdown options to a table using awk
awk -i inplace -v unsupported_mark="❌" '
function print_line() {
    if ($0 ~ /^#/) {
        print "#" $0
    }
    else {
        print
    }
}

function emit_option() {
    if (option == "") {
        return
    }

    gsub(/^[[:space:]]+|[[:space:]]+$/, "", description)
    gsub(/\|/, "\\|", option)
    gsub(/\|/, "\\|", description)
    print "| " option " | " description " | " unsupported_mark " |"
    option = ""
    description = ""
}

function begin_options() {
    if (in_commands) {
        end_commands()
        in_commands = 0
    }

    in_options = 1
    option_table = 0
    pending_blank = 0
    print_line()
}

function is_option_heading(value) {
    return value ~ /^(-|\*?\\?<|\*\*-)/
}

function start_option_table() {
    if (option_table) {
        return
    }

    print ""
    print "| Option | Description | Supported? |"
    print "| ------ | ----------- | ---------- |"
    option_table = 1
}

function emit_command() {
    if (command == "") {
        return
    }

    gsub(/^[[:space:]]+|[[:space:]]+$/, "", description)
    gsub(/\|/, "\\|", command)
    gsub(/\|/, "\\|", description)
    print "| " command " | " description " | " unsupported_mark " |"
    command = ""
    description = ""
}

function append_command_description(value) {
    sub(/^> ?/, "", value)

    if (done_description) {
        return
    }

    if (value == "") {
        done_description = 1
        return
    }

    if (description != "") {
        description = description " "
    }
    description = description value
}

function begin_commands(kind) {
    in_commands = 1
    command_kind = kind
    command_table = 0
    pending_command_blank = 0
    pending_plain_command = ""
    print_line()
}

function is_command_heading(value) {
    if (command_kind == "git") {
        return value ~ /^\*\*[^*]+\*\*\(1\)$/
    }

    return value ~ /^\*\*[^*]+\*\*([[:space:]].*)?$/
}

function is_plain_command_heading_start(value) {
    return command_kind == "subcommand" && value ~ /^[[:lower:]][[:alnum:]_-]*($|[[:space:]]+(\\\[|\\?<|--|-|\())/
}

function start_command_table() {
    if (command_table) {
        return
    }

    print ""
    print "| Command | Description | Supported? |"
    print "| ------- | ----------- | ---------- |"
    command_table = 1
}

function clean_command_heading(value) {
    if (command_kind == "git") {
        sub(/^\*\*/, "", value)
        sub(/\*\*\(1\)$/, "", value)
        sub(/^git-/, "", value)
    }

    return value
}

function end_commands() {
    emit_command()

    if (command_table) {
        print ""
        command_table = 0
    }
}

/^# NAME$/ {
    in_name = 1
    next
}

in_name && /^# / {
    print "# " name
    print ""
    in_name = 0
    print_line()
    next
}

in_name {
    if ($0 != "") {
        if (name != "") {
            name = name " "
        }
        name = name $0
    }

    next
}

in_options && /^# / {
    emit_option()
    print ""
    in_options = 0

    if ($0 ~ /^# (MODE )?OPTIONS$/) {
        begin_options()
        next
    }

    print_line()
    if ($0 == "# GIT COMMANDS") {
        in_commands = 1
        command_kind = "git"
        command_table = 0
    }
    next
}

/^# (MODE )?OPTIONS$/ {
    begin_options()
    next
}

in_options {
    if ($0 == "") {
        if (option_table) {
            next
        }

        pending_blank = 1
        next
    }

    if (is_option_heading($0)) {
        emit_option()
        pending_blank = 0
        start_option_table()
        option = $0
        done_description = 0
        next
    }

    if (!option_table) {
        if (pending_blank) {
            print ""
            pending_blank = 0
        }

        print
        next
    }

    sub(/^> ?/, "")

    if (done_description) {
        next
    }

    if ($0 == "") {
        done_description = 1
        next
    }

    if (description != "") {
        description = description " "
    }
    description = description $0

    next
}

/^# GIT COMMANDS$/ {
    begin_commands("git")
    next
}

/^# COMMANDS$/ {
    begin_commands("subcommand")
    next
}

in_commands && command == "" && !command_table && $0 == "" {
    pending_command_blank = 1
    next
}

in_commands && pending_command_blank && !is_command_heading($0) && !is_plain_command_heading_start($0) {
    print ""
    pending_command_blank = 0
}

in_commands && pending_plain_command != "" {
    if ($0 == "") {
        next
    }

    if ($0 ~ /^>/) {
        start_command_table()
        command = pending_plain_command
        pending_plain_command = ""
        done_description = 0
        append_command_description($0)
        next
    }

    pending_plain_command = pending_plain_command " " $0
    next
}

in_commands && is_command_heading($0) {
    emit_command()
    pending_command_blank = 0
    start_command_table()
    command = clean_command_heading($0)
    done_description = 0
    next
}

in_commands && command != "" {
    if ($0 == "") {
        next
    }

    if (is_plain_command_heading_start($0)) {
        emit_command()
        pending_command_blank = 0
        pending_plain_command = $0
        next
    }

    if ($0 !~ /^>/) {
        end_commands()
        print_line()
        next
    }

    sub(/^> ?/, "")
    append_command_description($0)

    next
}

in_commands && is_plain_command_heading_start($0) {
    emit_command()
    pending_command_blank = 0
    pending_plain_command = $0
    next
}

in_commands && command_table && $0 != "" {
    end_commands()
}

{
    print_line()
}

END {
    if (in_options) {
        emit_option()
    }

    if (in_commands) {
        end_commands()
    }
}
' "$2"
