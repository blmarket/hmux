I'm looking for a way to encapsulate entities.

1. entity expose defined trait
2. all interaction outside of the module should be done only via the trait
3. it may expose another traits when a single trait is insufficient to provide
   all functionality (e.g. WindowPane might be provided by Window trait, which
   looks reasonable)

Investigate existing entities and provide possible entity traits, ordered by
feasibility.
