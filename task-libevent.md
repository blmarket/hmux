This project is using libevent for scheduling, but it forces us to keep some
functions in `extern "C"`. I'd like to see whether the libevent's provide can
be abstracted and replaced.

As a first step of this, I'd like to abstract what libevent provides, so that
we can identify path forward to replace current libevent to different one which
does not require extern "C" function callbacks.

Create ./plan-libevent.md proposing candidate traits to abstract.

