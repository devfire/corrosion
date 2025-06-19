#!/usr/bin/env python3
"""
Test script for bandwidth throttling functionality.

This script tests the bandwidth throttling feature by:
1. Making HTTP requests through the proxy
2. Measuring download speeds
3. Verifying that bandwidth is properly limited

Usage:
    python3 test_bandwidth.py

Prerequisites:
    - Start the proxy with bandwidth throttling enabled:
      cargo run -- --dest-ip httpbin.org --dest-port 80 --bandwidth-enabled --bandwidth-limit-kbps 50

Expected behavior:
    - Download speeds should be limited to approximately 50 KB/s
    - Multiple requests should consistently show throttled speeds
"""

import time
import urllib.request
import urllib.error
import sys
from typing import Tuple

def test_bandwidth_throttling(proxy_url: str = "http://127.0.0.1:8080", 
                            test_size: int = 102400) -> Tuple[float, float]:
    """
    Test bandwidth throttling by downloading data and measuring speed.
    
    Args:
        proxy_url: URL of the proxy server
        test_size: Size of data to download in bytes
    
    Returns:
        Tuple of (download_time_seconds, speed_kbps)
    """
    # Use httpbin.org's /bytes endpoint to get a specific amount of data
    test_url = f"{proxy_url}/bytes/{test_size}"
    
    try:
        # Add Host header for httpbin.org
        req = urllib.request.Request(test_url)
        req.add_header('Host', 'httpbin.org')
        
        start_time = time.time()
        
        with urllib.request.urlopen(req, timeout=30) as response:
            data = response.read()
            
        end_time = time.time()
        
        download_time = end_time - start_time
        bytes_downloaded = len(data)
        speed_bps = bytes_downloaded / download_time
        speed_kbps = speed_bps / 1024
        
        return download_time, speed_kbps
        
    except urllib.error.URLError as e:
        print(f"Error connecting to proxy: {e}")
        return 0, 0
    except Exception as e:
        print(f"Unexpected error: {e}")
        return 0, 0

def main():
    print("=== Bandwidth Throttling Test ===")
    print()
    
    # Test configuration
    proxy_url = "http://127.0.0.1:8080"
    test_sizes = [51200, 102400, 204800]  # 50KB, 100KB, 200KB
    num_tests = 3
    
    print("Testing bandwidth throttling...")
    print(f"Proxy URL: {proxy_url}")
    print(f"Test sizes: {[size//1024 for size in test_sizes]} KB")
    print(f"Number of tests per size: {num_tests}")
    print()
    
    all_speeds = []
    
    for test_size in test_sizes:
        print(f"Testing with {test_size//1024} KB downloads:")
        
        speeds = []
        for i in range(num_tests):
            print(f"  Test {i+1}/{num_tests}...", end=" ")
            
            download_time, speed_kbps = test_bandwidth_throttling(proxy_url, test_size)
            
            if speed_kbps > 0:
                speeds.append(speed_kbps)
                all_speeds.append(speed_kbps)
                print(f"{download_time:.2f}s, {speed_kbps:.1f} KB/s")
            else:
                print("FAILED")
        
        if speeds:
            avg_speed = sum(speeds) / len(speeds)
            print(f"  Average speed: {avg_speed:.1f} KB/s")
        else:
            print("  All tests failed!")
        
        print()
    
    if all_speeds:
        overall_avg = sum(all_speeds) / len(all_speeds)
        min_speed = min(all_speeds)
        max_speed = max(all_speeds)
        
        print("=== Summary ===")
        print(f"Overall average speed: {overall_avg:.1f} KB/s")
        print(f"Speed range: {min_speed:.1f} - {max_speed:.1f} KB/s")
        print()
        
        # Check if bandwidth throttling is working
        expected_limit = 50  # Assuming 50 KB/s limit
        tolerance = 20  # 20% tolerance
        
        if overall_avg <= expected_limit * (1 + tolerance/100):
            print("✅ Bandwidth throttling appears to be working correctly!")
            if overall_avg <= expected_limit:
                print(f"   Speed is within the expected limit of {expected_limit} KB/s")
            else:
                print(f"   Speed is slightly above {expected_limit} KB/s but within tolerance")
        else:
            print("❌ Bandwidth throttling may not be working properly")
            print(f"   Expected speed around {expected_limit} KB/s, got {overall_avg:.1f} KB/s")
        
        print()
        print("Note: Actual speeds may vary due to:")
        print("- Network conditions")
        print("- Token bucket burst allowance")
        print("- HTTP protocol overhead")
        print("- System scheduling")
    else:
        print("❌ All tests failed! Make sure the proxy is running with bandwidth throttling enabled.")
        print()
        print("Start the proxy with:")
        print("cargo run -- --dest-ip httpbin.org --dest-port 80 --bandwidth-enabled --bandwidth-limit-kbps 50")

if __name__ == "__main__":
    main()