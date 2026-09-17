#!/usr/bin/env fish
#
# Takes a picture of every surface the editor has, into docs/previews.
#
#   tools/previews.fish                 # a new set, from a seed off the clock
#   tools/previews.fish --seed 1234     # the same set again
#   tools/previews.fish --check         # the release gate: are they current?
#
# A release ships screenshots and screenshots go stale silently, because nothing
# reads a picture. So --check is run before a release is cut. It fails when a
# picture is missing, when the code that draws them has uncommitted changes, and
# when the set was taken before the last commit that touched that code.
#
# It watches the code rather than the whole tree, and for the reason the obvious
# version does not work: the pictures are committed, so a stamp naming HEAD is
# stale the moment they are. What the stamp names is the last commit that touched
# crates/, which is what a picture can be out of date *with*, so committing the
# pictures, or editing this file, or writing a paragraph of documentation, leaves
# a current set current.
#
# It does not compare the images. The sound in them is random by design; what the
# gate holds is that somebody generated them from the code as it stands.
#
# The pictures are taken by the application itself, through its own view, with
# the window pointed at the library's simulated synthesizer. See
# crates/control/src/preview.rs.

set --local root (dirname (status --current-filename))/..
set --local out $root/docs/previews
set --local stamp $out/taken.txt
set --local seed ''
set --local checking 0

while set --query argv[1]
    switch $argv[1]
        case --check
            set checking 1
        case --seed
            set seed $argv[2]
            set --erase argv[1]
        case --out
            set out $argv[2]
            set stamp $out/taken.txt
            set --erase argv[1]
        case '*'
            echo "previews: $argv[1] is not one of --check, --seed, --out" >&2
            exit 2
    end
    set --erase argv[1]
end

# What a run would produce, asked of the tool rather than written out here: the
# pages are read off the library's own list of sections, so a group a later
# library adds gets a picture and gets checked for with nothing here to edit.
function expected --argument-names root
    cargo run --release --quiet --manifest-path $root/Cargo.toml --package control \
        --features previews --bin previews -- --list
end

# The last commit that changed anything the window is drawn out of.
function drawn_at --argument-names root
    git -C $root log -1 --format=%H -- crates
end

if test $checking -eq 1
    if not test -f $stamp
        echo "previews: none have been taken. Run tools/previews.fish" >&2
        exit 1
    end
    set --local missing 0
    for page in (expected $root)
        if not test -f $out/$page
            echo "previews: $page is missing" >&2
            set missing 1
        end
    end
    if test $missing -eq 1
        echo "previews: run tools/previews.fish before cutting a release" >&2
        exit 1
    end
    # A working tree with unsaved changes to the code cannot be photographed by
    # anything that reads git, so it is not asked to be: it is told to commit.
    if test (git -C $root status --porcelain -- crates | count) -gt 0
        echo "previews: crates/ has uncommitted changes; commit them first" >&2
        exit 1
    end
    set --local code (drawn_at $root)
    set --local taken (string match --regex --groups-only '^code (.+)$' < $stamp | head -1)
    if test "$taken" != "$code"
        echo "previews: taken at $taken, and the window was last changed at $code" >&2
        echo "previews: run tools/previews.fish before cutting a release" >&2
        exit 1
    end
    echo "previews: current with $code"
    exit 0
end

# A display, because the window is drawn by a graphics backend and a backend
# needs a surface to draw on. Whatever is already there is used, a desktop or
# somebody's own Xvfb, and one is started only when there is none, which is
# what makes this runnable on a machine with no screen.
set --local started ''
if not set --query DISPLAY
    if not command --query Xvfb
        echo "previews: no DISPLAY and no Xvfb to make one" >&2
        exit 1
    end
    set --global --export DISPLAY :99
    Xvfb :99 -screen 0 1700x1400x24 >/dev/null 2>&1 &
    set started $last_pid
    sleep 2
    # A screen nobody is looking at has no graphics card behind it, and what
    # wgpu does about that is spend a while finding out. The software rasterizer
    # is what draws these pictures on a machine like that; a desktop with a card
    # in it never reaches this, and anybody who has already chosen a backend
    # keeps theirs.
    if not set --query ICED_BACKEND
        set --global --export ICED_BACKEND tiny-skia
    end
end

mkdir -p $out

set --local arguments --out $out
if test -n "$seed"
    set arguments $arguments --seed $seed
end

# Optimised, and not as a preference: the pictures are drawn in software on any
# machine without a screen, and tiny-skia unoptimised takes minutes a frame
# where the same code takes milliseconds. A release tool that appears to hang is
# a release tool nobody runs.
set --local said (mktemp)
cargo run --release --quiet --manifest-path $root/Cargo.toml --package control \
    --features previews --bin previews -- $arguments 2>&1 | tee $said
set --local status_of_run $pipestatus[1]

if test -n "$started"
    kill $started 2>/dev/null
end

if test $status_of_run -ne 0
    rm -f $said
    echo "previews: the run failed" >&2
    exit $status_of_run
end

# The record beside the pictures: which state of the window they are of, what
# they were drawn from, and when. The first line is what --check reads; the rest
# is for whoever wants a particular set back.
set --local used (string match --regex --groups-only '^seed (.+)$' < $said | head -1)
begin
    echo "code "(drawn_at $root)
    echo "seed $used"
    echo "taken "(date --utc --iso-8601=seconds 2>/dev/null; or date -u +%Y-%m-%dT%H:%M:%S%z)
end > $stamp

rm -f $said
echo "previews: written to $out"
