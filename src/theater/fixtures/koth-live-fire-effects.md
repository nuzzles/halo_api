# KOTH Live Fire effect regressions

Match `9a87edc3-1758-46b2-ae6a-d248726dbcdf`.

- `koth-live-fire-detonation.bin`: first 17 bytes of chunk 3 packet payload at byte 687341, replay time 54.822579s. Event 5 occupies bits [1,129), body starts at 22; region 1, widths [11,11,10], quantized position [757,1374,502]. The final byte contains one event bit followed by neighboring data.
- `koth-live-fire-invalid-team.json`: contextual record at chunk 15 byte 717055 bits [1237,1320), replay time 291.926900s. The apparent team 6 comes from a mask containing out-of-schema bit 32 and must not replace Funky's team 2.
