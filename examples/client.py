import asyncio
import signal
from loguru import logger
from heartbeat_client import HeartbeatClient, setup_logging_bridge

# 将标准 logging 重定向到 loguru
setup_logging_bridge()


async def main():
    # 创建心跳客户端
    client = HeartbeatClient(
        server_host="chan.local",
        server_port=14999,
        heartbeat_interval=3,
        channel_id="test",
        timeout_message="(客户端: {client_address}) 已经 {time_since_heartbeat:.2f} 秒没有发送心跳",
        reconnect_interval=60
    )

    # 启动心跳
    logger.info("正在启动心跳客户端...")
    await client.start()

    # 设置信号处理
    loop = asyncio.get_event_loop()

    def signal_handler():
        logger.info("收到退出信号，正在停止心跳客户端...")
        client.stop()
        loop.stop()

    loop.add_signal_handler(signal.SIGINT, signal_handler)
    loop.add_signal_handler(signal.SIGTERM, signal_handler)

    # 保持运行
    logger.info("心跳客户端已启动，按 Ctrl+C 退出")

    try:
        while True:
            await asyncio.sleep(1)
    except asyncio.CancelledError:
        pass

if __name__ == "__main__":
    asyncio.run(main())
