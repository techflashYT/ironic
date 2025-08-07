from pyronic.client import *
from time import sleep

ipc = IPCClient()
esfd = ipc.IOSOpen("/dev/es")
ios_tid = 0x0000000100000000 | 58
print("fd={}".format(esfd))

num_views_ptr = ipc.alloc_raw(4)

res = ipc.IOSIoctlv(esfd, ES.GetNumTicketViews, "q:d", ios_tid, num_views_ptr)
if res < 0:
    print("ES_GetNumTicketViews() returned {}".format(res))
    ipc.IOSClose(esfd)
    ipc.shutdown()
    exit(0)

num_views = unpack(">I", num_views_ptr.read(off=0, size=4))[0]

SIZEOF_TIKVIEW = 0xd8
tikviews = ipc.alloc_raw(num_views * SIZEOF_TIKVIEW)

res = ipc.IOSIoctlv(esfd, ES.GetTicketViews, "qi:d", ios_tid, num_views, tikviews)
if res < 0:
    print("ES_GetTicketViews() returned {}".format(res))
    ipc.IOSClose(esfd)
    ipc.shutdown()
    exit(0)
ipc.IOSIoctlv(esfd, ES.LaunchTitle, "qd:", ios_tid, tikviews, noret=True)

# give it a bit
sleep(5)

# launch bc
esfd = ipc.IOSOpen("/dev/es")
msg = IPCMsg(IPCClient.IPC_IOCTLV, fd=esfd, args=[ES.LaunchBC])
ipc_buf = ipc.alloc_buf(msg.to_buffer())
ipc.sock.send_ipcmsg_noret(ipc_buf.paddr)

#ipc.debug_enable()
#ipc.shutdown()

#res = ipc.IOSIoctlv(esfd, ES.LaunchBC, "")
