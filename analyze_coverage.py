
import sys
import os

def parse_lcov(file_path):
    coverage = {}
    current_file = None
    
    try:
        with open(file_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line.startswith('SF:'):
                    current_file = line[3:]
                    if os.path.isabs(current_file):
                        # Try to make it relative to cwd
                        try:
                            current_file = os.path.relpath(current_file)
                        except ValueError:
                            pass
                    if current_file not in coverage:
                        coverage[current_file] = {'total': 0, 'covered': 0}
                elif line.startswith('DA:') and current_file:
                    parts = line[3:].split(',')
                    if len(parts) >= 2:
                        count = int(parts[1])
                        coverage[current_file]['total'] += 1
                        if count > 0:
                            coverage[current_file]['covered'] += 1
    except FileNotFoundError:
        print(f"Error: File {file_path} not found.")
        sys.exit(1)

    return coverage

def print_report(coverage):
    print(f"{'File':<60} | {'Lines':<10} | {'Covered':<10} | {'Coverage':<8}")
    print("-" * 95)
    
    total_lines = 0
    total_covered = 0
    
    sorted_files = sorted(coverage.keys())
    
    for file in sorted_files:
        stats = coverage[file]
        lines = stats['total']
        covered = stats['covered']
        percent = (covered / lines * 100) if lines > 0 else 0
        
        total_lines += lines
        total_covered += covered
        
        print(f"{file:<60} | {lines:<10} | {covered:<10} | {percent:6.2f}%")

    print("-" * 95)
    total_percent = (total_covered / total_lines * 100) if total_lines > 0 else 0
    print(f"{'TOTAL':<60} | {total_lines:<10} | {total_covered:<10} | {total_percent:6.2f}%")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python analyze_coverage.py <lcov_file>")
        sys.exit(1)
        
    lcov_file = sys.argv[1]
    coverage_data = parse_lcov(lcov_file)
    print_report(coverage_data)
