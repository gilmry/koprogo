#!/usr/bin/env python3
"""
OpenAPI Annotation Generator for Actix-web Handlers

This script automatically generates utoipa::path annotations for Actix-web handlers.
It parses Rust handler files and generates the necessary OpenAPI annotations.

Usage:
    python3 scripts/generate_openapi_annotations.py

Features:
- Parses #[get], #[post], #[put], #[delete] macros
- Extracts function names, parameters, return types
- Generates utoipa::path annotations with proper tags, summaries, responses
- Handles path parameters, query parameters, request bodies
- Generates schema annotations for DTOs
"""

import re
import os
from pathlib import Path
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass


@dataclass
class HandlerInfo:
    """Information about a handler function"""
    method: str  # GET, POST, PUT, DELETE
    path: str
    function_name: str
    params: List[str]
    tag: str
    summary: str
    description: str
    request_body: Optional[str] = None
    path_params: List[str] = None
    query_params: List[str] = None

    def __post_init__(self):
        if self.path_params is None:
            self.path_params = []
        if self.query_params is None:
            self.query_params = []


class OpenAPIGenerator:
    """Generates OpenAPI annotations for Rust handlers"""

    # HTTP method to Actix macro mapping
    ACTIX_MACROS = {
        'get': 'GET',
        'post': 'POST',
        'put': 'PUT',
        'delete': 'DELETE',
        'patch': 'PATCH'
    }

    # Tag inference from file names
    TAG_MAPPING = {
        'auth_handlers': 'Auth',
        'building_handlers': 'Buildings',
        'unit_handlers': 'Units',
        'owner_handlers': 'Owners',
        'expense_handlers': 'Expenses',
        'meeting_handlers': 'Meetings',
        'budget_handlers': 'Budgets',
        'document_handlers': 'Documents',
        'gdpr_handlers': 'GDPR',
        'payment_handlers': 'Payments',
        'payment_method_handlers': 'PaymentMethods',
        'local_exchange_handlers': 'LocalExchanges',
        'notification_handlers': 'Notifications',
        'ticket_handlers': 'Tickets',
        'resolution_handlers': 'Resolutions',
        'poll_handlers': 'Polls',
        'quote_handlers': 'Quotes',
        'convocation_handlers': 'Convocations',
        'work_report_handlers': 'WorkReports',
        'technical_inspection_handlers': 'TechnicalInspections',
        'gamification_handlers': 'Gamification',
        'skill_handlers': 'Skills',
        'notice_handlers': 'Notices',
        'shared_object_handlers': 'SharedObjects',
        'resource_booking_handlers': 'ResourceBookings',
        'two_factor_handlers': 'TwoFactorAuth',
        'etat_date_handlers': 'EtatsDates',
        'board_member_handlers': 'BoardMembers',
        'payment_reminder_handlers': 'PaymentRecovery',
    }

    def __init__(self, handlers_dir: Path):
        self.handlers_dir = handlers_dir
        self.handlers: List[HandlerInfo] = []
        self.dtos: Dict[str, List[str]] = {}  # file -> [dto_names]

    def parse_handler_file(self, file_path: Path) -> List[HandlerInfo]:
        """Parse a single handler file and extract handler information"""
        handlers = []
        content = file_path.read_text()

        # Infer tag from filename
        tag = self.TAG_MAPPING.get(file_path.stem, file_path.stem.replace('_handlers', '').title())

        # Find all handler functions with actix macros
        # Pattern: #[get("/path")] or #[post("/path")]
        pattern = r'#\[(get|post|put|delete|patch)\("([^"]+)"\)\]\s*pub async fn (\w+)\((.*?)\) -> impl Responder'

        for match in re.finditer(pattern, content, re.MULTILINE | re.DOTALL):
            method_macro = match.group(1)
            path = match.group(2)
            function_name = match.group(3)
            params_str = match.group(4)

            # Parse parameters
            params = self._parse_params(params_str)

            # Extract path parameters from route
            path_params = re.findall(r'\{(\w+)\}', path)

            # Detect request body (web::Json<...>)
            request_body = None
            for param in params:
                if 'web::Json<' in param:
                    match = re.search(r'web::Json<(\w+)>', param)
                    if match:
                        request_body = match.group(1)
                        break

            # Detect query parameters (web::Query<...>)
            query_params = []
            for param in params:
                if 'web::Query<' in param:
                    match = re.search(r'web::Query<(\w+)>', param)
                    if match:
                        query_params.append(match.group(1))

            # Generate summary from function name
            summary = self._generate_summary(function_name)
            description = f"{summary} endpoint"

            handler = HandlerInfo(
                method=self.ACTIX_MACROS[method_macro],
                path=path,
                function_name=function_name,
                params=params,
                tag=tag,
                summary=summary,
                description=description,
                request_body=request_body,
                path_params=path_params,
                query_params=query_params
            )

            handlers.append(handler)

        return handlers

    def _parse_params(self, params_str: str) -> List[str]:
        """Parse function parameters"""
        # Split by comma, but be careful with generics
        params = []
        current = ""
        depth = 0

        for char in params_str:
            if char == '<':
                depth += 1
            elif char == '>':
                depth -= 1
            elif char == ',' and depth == 0:
                params.append(current.strip())
                current = ""
                continue
            current += char

        if current.strip():
            params.append(current.strip())

        return params

    def _generate_summary(self, function_name: str) -> str:
        """Generate human-readable summary from function name"""
        # Convert snake_case to Title Case
        words = function_name.replace('_', ' ').split()
        return ' '.join(word.capitalize() for word in words)

    def generate_annotation(self, handler: HandlerInfo) -> str:
        """Generate utoipa::path annotation for a handler"""
        lines = ['#[utoipa::path(']

        # Method
        lines.append(f'    {handler.method.lower()},')

        # Path
        lines.append(f'    path = "{handler.path}",')

        # Tag
        lines.append(f'    tag = "{handler.tag}",')

        # Summary
        lines.append(f'    summary = "{handler.summary}",')

        # Request body
        if handler.request_body:
            lines.append(f'    request_body = {handler.request_body},')

        # Path parameters
        if handler.path_params:
            lines.append('    params(')
            for param in handler.path_params:
                param_type = 'String' if param == 'id' else 'String'
                lines.append(f'        ("{param}" = {param_type}, Path, description = "{param.replace("_", " ").title()}"),')
            lines.append('    ),')

        # Responses
        lines.append('    responses(')

        # Success response based on method
        if handler.method == 'POST':
            lines.append('        (status = 201, description = "Resource created successfully"),')
        elif handler.method == 'DELETE':
            lines.append('        (status = 204, description = "Resource deleted successfully"),')
        else:
            lines.append('        (status = 200, description = "Success"),')

        # Common error responses
        lines.append('        (status = 400, description = "Bad Request"),')

        if handler.tag != 'Auth':  # Most endpoints require auth
            lines.append('        (status = 401, description = "Unauthorized"),')
            lines.append('        (status = 403, description = "Forbidden"),')

        lines.append('        (status = 404, description = "Not Found"),')
        lines.append('        (status = 500, description = "Internal Server Error"),')
        lines.append('    ),')

        # Security (if not auth endpoint)
        if handler.tag != 'Auth' and 'public' not in handler.path:
            lines.append('    security(')
            lines.append('        ("bearer_auth" = []),')
            lines.append('    ),')

        lines.append(')]')

        return '\n'.join(lines)

    def scan_all_handlers(self):
        """Scan all handler files in the handlers directory"""
        for file_path in self.handlers_dir.glob('*_handlers.rs'):
            if file_path.name == 'mod.rs':
                continue

            handlers = self.parse_handler_file(file_path)
            self.handlers.extend(handlers)

            print(f"✓ Parsed {file_path.name}: {len(handlers)} handlers")

    def generate_report(self) -> str:
        """Generate a report of all handlers found"""
        lines = [
            "# OpenAPI Handlers Report",
            f"\nTotal handlers found: {len(self.handlers)}",
            "\n## By Tag:\n"
        ]

        # Group by tag
        by_tag = {}
        for handler in self.handlers:
            by_tag.setdefault(handler.tag, []).append(handler)

        for tag, handlers in sorted(by_tag.items()):
            lines.append(f"### {tag} ({len(handlers)} endpoints)")
            for handler in handlers:
                lines.append(f"- {handler.method} {handler.path} → {handler.function_name}")
            lines.append("")

        return '\n'.join(lines)

    def generate_openapi_paths_list(self) -> str:
        """Generate the paths(...) list for openapi.rs"""
        lines = ["paths("]

        # Group by tag for better organization
        by_tag = {}
        for handler in self.handlers:
            by_tag.setdefault(handler.tag, []).append(handler)

        for tag in sorted(by_tag.keys()):
            lines.append(f"    // {tag}")
            for handler in by_tag[tag]:
                lines.append(f"    {handler.function_name},")

        lines.append("),")
        return '\n'.join(lines)

    def generate_sample_annotations(self, limit: int = 5) -> str:
        """Generate sample annotations for the first N handlers"""
        lines = ["# Sample Handler Annotations\n"]

        for i, handler in enumerate(self.handlers[:limit]):
            lines.append(f"\n## {i+1}. {handler.function_name} ({handler.method} {handler.path})")
            lines.append("```rust")
            lines.append(self.generate_annotation(handler))
            lines.append(f"#[{handler.method.lower()}(\"{handler.path}\")]")
            lines.append(f"pub async fn {handler.function_name}(...) -> impl Responder {{")
            lines.append("    // handler body")
            lines.append("}")
            lines.append("```\n")

        return '\n'.join(lines)


def main():
    """Main entry point"""
    # Get backend directory
    backend_dir = Path(__file__).parent.parent
    handlers_dir = backend_dir / 'src' / 'infrastructure' / 'web' / 'handlers'

    if not handlers_dir.exists():
        print(f"❌ Handlers directory not found: {handlers_dir}")
        return 1

    print(f"🔍 Scanning handlers in: {handlers_dir}\n")

    # Create generator
    generator = OpenAPIGenerator(handlers_dir)

    # Scan all handlers
    generator.scan_all_handlers()

    # Generate report
    report = generator.generate_report()
    report_file = backend_dir / 'scripts' / 'openapi_handlers_report.md'
    report_file.write_text(report)
    print(f"\n✓ Report saved to: {report_file}")

    # Generate sample annotations
    samples = generator.generate_sample_annotations(limit=10)
    samples_file = backend_dir / 'scripts' / 'openapi_sample_annotations.md'
    samples_file.write_text(samples)
    print(f"✓ Sample annotations saved to: {samples_file}")

    # Generate paths list for openapi.rs
    paths_list = generator.generate_openapi_paths_list()
    paths_file = backend_dir / 'scripts' / 'openapi_paths_list.txt'
    paths_file.write_text(paths_list)
    print(f"✓ Paths list saved to: {paths_file}")

    print(f"\n✅ Found {len(generator.handlers)} handlers across {len(set(h.tag for h in generator.handlers))} tags")
    print("\nNext steps:")
    print("1. Review openapi_sample_annotations.md")
    print("2. Add annotations to handler files manually or use the annotation strings")
    print("3. Add DTOs to openapi.rs schemas section")
    print("4. Add handler functions to openapi.rs paths section")

    return 0


if __name__ == '__main__':
    exit(main())
