See agentmon-tui/ for context.

I was working at /srv/hmux, which is parent directory of here, and started a new
job from there.

I expected the job to create worktree at /srv/, such as /srv/wt/, but I found
/srv/hmux/wt/ was created. Seems like the app is following CWD resolving the items.

Also new job creation dialog shows that the source repository is /srv/hmux/hmux/
which is not my intention.

I want worktree creation should follow the repository the command has been
requested (e.g. I pressed 'r' when the cursor was on /srv/hmux/ repository, and
I wanted it to be honored)

