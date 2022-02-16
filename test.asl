disc(small, 1).
disc(med, 2).
disc(large, 3).

on(large, 0, table).
on(med, 0, large).
on(small, 1, table).

+!sort : on(Disc, _, table) <-
  .print(Disc).
