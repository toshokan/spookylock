# spookylock

A simple but secure screen locker. 

## Design

Unlike most simple screen lockers, spookylock works in text or
graphical mode and prevents switching VTs when it is active. This
design was inspired by
[physlock](https://github.com/muennich/physlock).

Unlike physlock, spookylock uses a (very simple) TUI interface rather
than a simple password prompt. It decides which user's password will
unlock the session through the `$USER` environment variable, rather
than using `systemd` or `elogind`. This means it can be invoked on
behalf of other system users. It uses the standard `system-auth` PAM
config.

Spookylock is designed with a modular architecture. It is used through
two executables, `spookylock` and `spookylock-interface`. To lock a
session, users invoke `spookylock`. This program's responsibility is
to switch to a new blank controlling VT/TTY and ensure that the user
cannot leave.

It launches `spookylock-interface` on that VT to do the actual IO with
the user. This program initializes the TUI, draws the interface, asks
for the credentials, and communicates with PAM.

Once `spookylock-interface` exits successfully, suggesting a
successful login, the parent process unlocks the VT/TTY, cleans up and
returns the user to their session.

This means users can write users can write their own programs to set
up whichever interface or authentication scheme they like (or do
something that has nothing to do with authentication) The program will
be invoked as `yourprogram --user <user>` where `<user>` is the
contents of the `$USER` variable when `spookylock` is invoked.

If you want to change which program `spookylock` invokes you can use
its command line options. If you want to use another program on
`$PATH`, invoke it as `spookylock -i <yourprogram>`. If you have a
(relative or absolute) path to an interface program instead, invoke it
as `spookylock -r -i <path/to/yourprogram>`

## Installation

Compile the workspace with `cargo build --release`.

Copy `target/release/spookylock` and
`target/release/spookylock-interface` to somewhere on `$PATH`. Ensure
that `spookylock` is invoked with effective root privileges (it has to
access `/dev/console/`). You can do this either with sudo (perhaps
make a sudo rule to do this passwordless if you want to be able to
lock quickly), or set the setuid bit (`chmod +s /path/to/spookylock`)

## License

Spookylock is dual licensed under the MIT license and Apache 2.0 license.
You may choose which one you prefer.
