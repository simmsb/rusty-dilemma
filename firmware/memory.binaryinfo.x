SECTIONS {
    /* ### Picotool 'Binary Info' Header Block
     *
     * Picotool only searches the second 256 bytes of Flash for this block, but
     * that's where our vector table is. We squeeze in this block after the
     * vector table, but before .text.
     */
    .bi_header : ALIGN(4)
    {
        KEEP(*(.bi_header));
        /* Keep this block a nice round size */
        . = ALIGN(4);
    } > FLASH
} INSERT BEFORE .text;

/* Move _stext, to make room for our new section */
_stext = ADDR(.bi_header) + SIZEOF(.bi_header);

SECTIONS {
    /* ### Picotool 'Binary Info' Entries
     *
     * Picotool looks through this block (as we have pointers to it in our header) to find interesting information.
     */
    .bi_entries : ALIGN(4)
    {
        /* We put this in the header */
        __bi_entries_start = .;
        /* Here are the entries */
        KEEP(*(.bi_entries));
        /* Keep this block a nice round size */
        . = ALIGN(4);
        /* We put this in the header */
        __bi_entries_end = .;
    } > FLASH
} INSERT AFTER .text;
