Check all writer of Buf struct, see the writers can be just Bytes or BytesMut
and move them to Buf (instead of memory copy).
