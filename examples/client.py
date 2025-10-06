import asyncio
from heartbeat_client import HeartbeatClient, setup_logging_bridge
from loguru import logger
from pydantic import BaseModel


class HeartBeatConfig(BaseModel):
    """客户端配置"""

    server_host: str = "chan.local"
    server_port: int = 14999
    heartbeat_interval: int = 60  # 秒
    channel_id: str = "gp_factory"
    timeout_message: str = "gp_factory可能已停止运行(客户端: {client_address}) "
    reconnect_interval: int = 60


async def _heartbeat_task(config: HeartBeatConfig):
    setup_logging_bridge()
    hb_client = HeartbeatClient(
        server_host=config.server_host,
        server_port=config.server_port,
        heartbeat_interval=config.heartbeat_interval,
        channel_id=config.channel_id,
        timeout_message=config.timeout_message,
        reconnect_interval=config.reconnect_interval,
    )
    try:
        await hb_client.start()
    except Exception as e:
        logger.error(f"心跳客户端异常: {e}")


def run_heartbeat(config: HeartBeatConfig):
    logger.info("正在启动心跳客户端...")
    loop = asyncio.get_event_loop()
    _ = loop.create_task(_heartbeat_task(config))
    logger.info("心跳客户端已在后台启动")


async def main():
    run_heartbeat(HeartBeatConfig())
    try:
        while True:
            await asyncio.sleep(1)
    except KeyboardInterrupt:
        logger.info("程序已退出")


if __name__ == "__main__":
    asyncio.run(main())

