"""HTML block and table extraction helpers"""

from dataclasses import dataclass
from html.parser import HTMLParser
from urllib.request import Request, urlopen

from fruitfacts_extract.text_tools import DEFAULT_USER_AGENT, clean_text


@dataclass
class HtmlPage:
    blocks: list
    tables: list


class HtmlBlockTableParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.blocks = []
        self.tables = []
        self.current = None
        self.skip_depth = 0
        self.in_table = False
        self.current_row = None
        self.current_cell = None

    def handle_starttag(self, tag, attrs):
        if tag in ("script", "style", "noscript"):
            self.skip_depth += 1
            return
        if self.skip_depth:
            return

        if tag == "table":
            self.in_table = True
            self.tables.append([])
            return
        if self.in_table:
            if tag == "tr":
                self.current_row = []
            elif tag in ("th", "td") and self.current_row is not None:
                self.current_cell = []
            elif tag == "br" and self.current_cell is not None:
                self.current_cell.append(" ")
            return

        if tag in ("h1", "h2", "h3", "h4", "p", "li"):
            self.current = [tag, []]
        elif self.current and tag == "br":
            self.current[1].append(" ")

    def handle_endtag(self, tag):
        if tag in ("script", "style", "noscript"):
            self.skip_depth = max(0, self.skip_depth - 1)
            return
        if self.skip_depth:
            return

        if self.in_table:
            if tag in ("th", "td") and self.current_cell is not None:
                text = clean_text("".join(self.current_cell))
                self.current_row.append(text)
                self.current_cell = None
            elif tag == "tr" and self.current_row is not None:
                if any(self.current_row):
                    self.tables[-1].append(self.current_row)
                self.current_row = None
            elif tag == "table":
                self.in_table = False
            return

        if self.current and tag == self.current[0]:
            text = clean_text("".join(self.current[1]))
            if text:
                self.blocks.append((tag, text))
            self.current = None

    def handle_data(self, data):
        if self.skip_depth:
            return
        if self.current_cell is not None:
            self.current_cell.append(data)
        elif self.current:
            self.current[1].append(data)


def fetch_html_page(url, user_agent=DEFAULT_USER_AGENT, timeout=45):
    request = Request(url, headers={"User-Agent": user_agent})
    html = urlopen(request, timeout=timeout).read().decode("utf-8", "replace")
    parser = HtmlBlockTableParser()
    parser.feed(html)
    return HtmlPage(blocks=parser.blocks, tables=parser.tables)


def blocks_between_headings(blocks, start_heading, end_heading=None):
    start_index = None
    for index, (tag, text) in enumerate(blocks):
        if tag.startswith("h") and text == start_heading:
            start_index = index + 1
            break
    if start_index is None:
        raise ValueError("Could not find heading: " + start_heading)

    end_index = len(blocks)
    if end_heading:
        for index in range(start_index, len(blocks)):
            tag, text = blocks[index]
            if tag.startswith("h") and text == end_heading:
                end_index = index
                break
        if end_index == len(blocks):
            raise ValueError("Could not find heading: " + end_heading)
    return blocks[start_index:end_index]
