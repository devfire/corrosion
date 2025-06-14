#!/usr/bin/env python3
"""
Simple test script to demonstrate latency fault injection.
This script makes HTTP requests through the proxy and measures response times.
"""

import time
import socket
import ssl
import sys

def test_connection_with_timing(host, port, num_requests=3):
    """Test connection through proxy and measure timing"""
    print(f"Testing connection to {host}:{port}")
    print(f"Making {num_requests} requests...\n")
    
    times = []
    
    for i in range(num_requests):
        start_time = time.time()
        
        try:
            # Create socket connection through proxy
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.settimeout(10)
            
            # Connect to proxy
            sock.connect((host, port))
            
            # Wrap with SSL for HTTPS
            context = ssl.create_default_context()
            ssl_sock = context.wrap_socket(sock, server_hostname='httpbin.org')
            
            # Send HTTP request
            request = (
                "GET /get HTTP/1.1\r\n"
                "Host: httpbin.org\r\n"
                "Connection: close\r\n"
                "\r\n"
            )
            ssl_sock.send(request.encode())
            
            # Read response (just the first part)
            response = ssl_sock.recv(1024)
            
            ssl_sock.close()
            
            end_time = time.time()
            duration = (end_time - start_time) * 1000  # Convert to milliseconds
            times.append(duration)
            
            print(f"Request {i+1}: {duration:.2f}ms")
            
        except Exception as e:
            print(f"Request {i+1} failed: {e}")
            
        time.sleep(0.5)  # Small delay between requests
    
    if times:
        avg_time = sum(times) / len(times)
        print(f"\nAverage response time: {avg_time:.2f}ms")
        print(f"Min: {min(times):.2f}ms, Max: {max(times):.2f}ms")
    
    return times

if __name__ == "__main__":
    host = "127.0.0.1"
    port = 8081
    
    if len(sys.argv) > 1:
        port = int(sys.argv[1])
    
    print("=== Latency Fault Injection Test ===")
    print(f"Proxy should be running on {host}:{port}")
    print("Expected latency: 500ms fixed + 100-300ms random = 600-800ms total\n")
    
    test_connection_with_timing(host, port, 5)