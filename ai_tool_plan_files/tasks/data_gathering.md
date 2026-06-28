a primary goal of the project is to be an index for high-quality variety data sources such as land grant university growing guides, books, published variety comparison studies, etc.  I was initially gathering these by hand and then importing them by hand.  I'd like to develop a set of tools:

1. to search for new sources using AI search assistance
2. to download and index them initially with blank .json5 files with "needs_help: true"
3. to parse them, with text recognition or image recognition, into our plant_database/... .json5 files
4. to go through the "help needed" backlog and discern which updates are needed (typically, just delayed parsing) and perform those updates
5. to sanity-check our data sources in various ways

I envision small guide/doc files that describe these concrete tasks:
1. compare a given reference (usually pdf) with its .json5 encoding/index file. learn from its associations and document the typical ways associations are set up in a helper file (a single helper doc for all work, not one per reference, that gets updated as a guide to other steps)
2. update a given .json5 encoding/index file with new/missing data from a given reference
3. perform a web search for new high-quality data sources. the categories of data sources should mirror those already in existence. the output should be a list of candidate files. the inputs are, let's say, google search, bing, AI search, etc.  For each data source found, find a concrete downloadable URL
4. survey existing references for *their* references/citations, and look those up, and add them to the list (with concrete downloadable URLs)
5. step through the list of downloadable URLs and download them, and sort them by location or type as has been done for existing references
6. do greenfield encoding/index file creation from these references

immediate tasks:
1. I'd like you to start work on concise docs for each of the above tasks
2. please think about expanding the list of concrete tasks
3. for each concrete task, create a placeholder guide doc and add that to an overall helper file (this file)
