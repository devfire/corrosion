#!/usr/bin/env python3
"""
Test script for packet loss simulation in the fault injection proxy.
This script sends multiple HTTP requests and measures packet loss rates.
"""

import asyncio
import aiohttp
import time
import statistics
from typing import List, Tuple

async def send_request(session: aiohttp.ClientSession, url: str, timeout: int = 5) -> Tuple[bool, float]:
    """Send a single HTTP request and return (success, response_time)"""
    start_time = time.time()
    try:
        async with session.get(url, timeout=aiohttp.ClientTimeout(total=timeout)) as response:
            await response.text()  # Read the response body
            end_time = time.time()
            return True, end_time - start_time
    except Exception as e:
        end_time = time.time()
        print(f"Request failed: {e}")
        return False, end_time - start_time

async def test_packet_loss(proxy_url: str, num_requests: int = 100, concurrent: int = 10) -> None:
    """Test packet loss by sending multiple requests through the proxy"""
    print(f"Testing packet loss with {num_requests} requests ({concurrent} concurrent)")
    print(f"Target URL: {proxy_url}")
    print("-" * 60)
    
    connector = aiohttp.TCPConnector(limit=concurrent * 2)
    timeout = aiohttp.ClientTimeout(total=10)
    
    async with aiohttp.ClientSession(connector=connector, timeout=timeout) as session:
        # Create semaphore to limit concurrent requests
        semaphore = asyncio.Semaphore(concurrent)
        
        async def bounded_request():
            async with semaphore:
                return await send_request(session, proxy_url)
        
        # Send all requests
        start_time = time.time()
        tasks = [bounded_request() for _ in range(num_requests)]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        end_time = time.time()
        
        # Process results
        successful_requests = 0
        failed_requests = 0
        response_times = []
        
        for result in results:
            if isinstance(result, Exception):
                failed_requests += 1
            else:
                success, response_time = result
                if success:
                    successful_requests += 1
                    response_times.append(response_time)
                else:
                    failed_requests += 1
        
        # Calculate statistics
        total_time = end_time - start_time
        success_rate = (successful_requests / num_requests) * 100
        packet_loss_rate = (failed_requests / num_requests) * 100
        
        print(f"Results:")
        print(f"  Total requests: {num_requests}")
        print(f"  Successful: {successful_requests}")
        print(f"  Failed: {failed_requests}")
        print(f"  Success rate: {success_rate:.1f}%")
        print(f"  Packet loss rate: {packet_loss_rate:.1f}%")
        print(f"  Total time: {total_time:.2f}s")
        print(f"  Requests/second: {num_requests/total_time:.1f}")
        
        if response_times:
            print(f"  Response time stats:")
            print(f"    Min: {min(response_times)*1000:.1f}ms")
            print(f"    Max: {max(response_times)*1000:.1f}ms")
            print(f"    Mean: {statistics.mean(response_times)*1000:.1f}ms")
            print(f"    Median: {statistics.median(response_times)*1000:.1f}ms")

async def test_different_loss_rates():
    """Test different packet loss configurations"""
    test_cases = [
        ("No packet loss", "http://127.0.0.1:8080"),
        ("10% packet loss", "http://127.0.0.1:8081"),
        ("25% packet loss", "http://127.0.0.1:8082"),
        ("50% packet loss", "http://127.0.0.1:8083"),
    ]
    
    for name, url in test_cases:
        print(f"\n{'='*60}")
        print(f"Testing: {name}")
        print(f"{'='*60}")
        try:
            await test_packet_loss(url, num_requests=50, concurrent=5)
        except Exception as e:
            print(f"Test failed: {e}")
        
        # Wait between tests
        await asyncio.sleep(2)

def main():
    """Main function"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Test packet loss simulation")
    parser.add_argument("--url", default="http://127.0.0.1:8080", 
                       help="Proxy URL to test (default: http://127.0.0.1:8080)")
    parser.add_argument("--requests", type=int, default=100,
                       help="Number of requests to send (default: 100)")
    parser.add_argument("--concurrent", type=int, default=10,
                       help="Number of concurrent requests (default: 10)")
    parser.add_argument("--test-all", action="store_true",
                       help="Test multiple packet loss rates")
    
    args = parser.parse_args()
    
    if args.test_all:
        asyncio.run(test_different_loss_rates())
    else:
        asyncio.run(test_packet_loss(args.url, args.requests, args.concurrent))

if __name__ == "__main__":
    main()