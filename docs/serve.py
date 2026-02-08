import http.server
import socketserver
import os
import sys

PORT = 8089
DIRECTORY = os.path.dirname(os.path.abspath(__file__))

class CORSRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, OPTIONS')
        self.send_header("Access-Control-Allow-Headers", "X-Requested-With, Content-Type")
        
        # Explicit Content-Type for GeoJSON
        if self.path.endswith(".geojson"):
            self.send_header("Content-Type", "application/geo+json")
            
        super().end_headers()

    def do_OPTIONS(self):
        self.send_response(200, "ok")
        self.end_headers()

if __name__ == "__main__":
    # Change to the script's directory (docs/)
    os.chdir(DIRECTORY)
    
    # Allow port override
    if len(sys.argv) > 1:
        try:
            PORT = int(sys.argv[1])
        except ValueError:
            pass

    print(f"Serving docs from {DIRECTORY} on port {PORT} with CORS...")
    with socketserver.TCPServer(("", PORT), CORSRequestHandler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            pass
