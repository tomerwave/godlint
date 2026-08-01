# Accepted released drift

- `ci/no-monolithic-job` — relaxed rule: Across 231 real jobs, the median was 6 steps and 36% exceeded the old threshold of 7, so #233 raised `max-steps` to 20.

Delete this file in the first pull request after the release publishes. Until then the declaration is
still true: the released binary a pull request is measured against is the previous one, so a release
pull request that deleted this file would fail its own released-agreement check.
