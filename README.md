# woof-rs

A [woof](http://www.home.unix-ag.org/simon/woof.html) clone.

# how to use it

Simply build, pass in some filenames into the arguments, execute, and give your
friend(s) the link it spits out!

Passing in the `--help` flag will list all the options you can tweak.

# modes

## default mode

The default mode. Hosts any files you give it.

A user can either request `host/file` manually, or GET `host/` to receive a
small HTML document that will automagically download all given.

## upload mode

Specified with `--upload`/`-U`.

On GET `host/`, the user will receive a small HTML document containing a file
upload form. Once they hit the `Submit` button, the files will be transferred and
downloaded onto the server's computer.

If you want to be fancy, you can also PUT `host/upload` with the `File-Name`
header set to the name of the file, and the body set to its contents.
