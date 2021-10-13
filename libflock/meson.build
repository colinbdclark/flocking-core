project('libflock', 'c')
project_description = 'A very portable audio library.'

headers = include_directories('include')

source_files = [
    'src/libflock.c'
]

# libflock
libflock_library = static_library(
    meson.project_name(),
    source_files
)

libflock_dep = declare_dependency(
    include_directories: headers,
    link_with: libflock_library
)

# Examples
executable(
    'libflock-console-example',
    'examples/console/src/print-silence.c',
    dependencies: [libflock_dep]
)
