from pyronic.client import *

ipc = IPCClient()
esfd = ipc.IOSOpen("/dev/es")
print(f"ES FD: {esfd}")

msg = IPCMsg(IPCClient.IPC_IOCTLV, fd=esfd, args=[ES.LaunchBC])
ipc_buf = ipc.alloc_buf(msg.to_buffer())
ipc.sock.send_ipcmsg_noret(ipc_buf.paddr)

#ipc.debug_enable()
#ipc.shutdown()

#res = ipc.IOSIoctlv(esfd, ES.LaunchBC, "")
