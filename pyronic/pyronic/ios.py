from enum import IntEnum
from struct import pack, unpack

class IOSErr(IntEnum):
    FS_EINVAL       = -4
    FS_EACCESS      = -102
    FS_ENOENT       = -106
    ES_EINVAL       = -1017


class ES(IntEnum):
    AddTicket       = 0x01
    AddTitleStart   = 0x02
    AddContentStart = 0x03
    AddContentData  = 0x04
    AddContentFinish= 0x05
    AddTitleFinish  = 0x06
    LaunchTitle     = 0x08
    GetTitlesCount  = 0x0e
    GetNumTicketViews  = 0x12
    GetTicketViews  = 0x13
    GetTitles       = 0x0f
    AddTitleCancel  = 0x2f
    LaunchBC        = 0x25

class AES(IntEnum):
    Copy = 0x0
    Encrypt = 0x02
    Decrypt = 0x03

class SDIO(IntEnum):
    Write8 = 0x1
    Read8 = 0x2
    ResetCard = 0x4
    SetClock = 0x6
    Command = 0x7
    GetStatus = 0xb
    GetOCR = 0xc

class IPCMsg(object):
    """ A structure representing some PPC-to-ARM IPC message. 
    After this is filled out, the user will obtain the raw bytes and write 
    them to physical memory somewhere (aligned to 32-byte boundaries).
    """
    def __init__(self, cmd, fd=0, args=[0,0,0,0,0]):
        self.cmd = cmd
        self.res = 0
        self.fd = fd
        self.args = args

    def to_buffer(self):
        """ Convert to a big-endian binary representation """
        while len(self.args) < 5: 
            self.args.append(0)
        assert len(self.args) == 5
        return pack(">Lii5L", self.cmd, self.res, self.fd, *self.args)


