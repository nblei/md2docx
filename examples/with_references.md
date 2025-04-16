---
title: Document with Image References
author: Nathan Bleier
---

# Document with Images and References

This document demonstrates the image reference feature.

Here's our university logo (small):

![{"scale":0.5,"ref":"logo-small"}](../data/University-of-Michigan-Logo.png "University of Michigan")

As you can see in {ref:logo-small}, the logo is scaled down to 50%.

Here's our university logo (large):

![{"scale":1.0,"ref":"logo-large"}](../data/University-of-Michigan-Logo.png "University of Michigan")

As mentioned in {ref:logo-large}, this is the full size logo.

## References section

Both {ref:logo-small} and {ref:logo-large} are versions of the same image with different scaling.

Some references to a non-existent image: {ref:missing} should not be replaced.
