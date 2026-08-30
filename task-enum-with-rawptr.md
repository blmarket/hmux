There are enums where variants has *mut ptr. I'm wondering they can be migrated
to Box if the ownership is owned by enum and some free callback will be called.

One example is OverlayData.

Check the situation and find other candidates as ./report-enum-with-rawptr.md
