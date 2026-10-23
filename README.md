# Code and Data for [www.fruitfacts.xyz](https://www.fruitfacts.xyz)
* fruitfacts is a website to track typical harvest times for crops and help gardeners achieve successive ripening of varieties that work well in their locations
* it consists of a cross-referencing system for common tree fruits, with an emphasis on information from university agricultural extension publications and other evidence-based sources, as well as tools like searching by location or various filters, and harvest charting
* fruitfacts has an emphasis on citations rather than on editorializing or paraphrasing other works; the original content consists mainly of calculated values, the cross-referencing system, search tools, and so on

* [Backend README](backend/README.md)
  * written in rust using a sqlite database
* [Frontend README](frontend/README.md)
  * written in javascript with next.js and react

# contributing
* The plant database is based on imported publications.  Please search in your local area and see if any useful publications aren't yet imported or look at this list:
  * [a list of data files that need help](plant_database/help_needed.md)
* data cleanup - naming problems like duplicate entries under multiple spellings, transcription errors, etc.
* I need software help too.  If you have ideas, there are links to browse the relevant code on each page. see also this list:
  * [help wanted code projects](help%20wanted.md)

## goals
* able to reproduce all of the common charts like DWN, ACN, and charts in university extension publications with some level of beauty. charts can be private or public and can be saved to a permalink
* an extensive plant database with harvest dates and references that users can start with, pull into their own charts and then modify. when adding new varieties, a default harvest time should be suggested based on the closest available data or some formula
* each variety's page should contain a list of references with harvest dates and also dates from users if they've set their data to be public
* support a few methods for harvest windows: day of year ranges, relative start like "redhaven+5", or "early/mid/late" (rated in % through the season)
* users shouldn't need to stick to the existing plant database when creating their own charts, but the existing one should be selected as a linked variety whenever possible to help share data and provide good references
* the plant database should be in a simple text format and hosted on github so it can be shared and extended by semi-technical users without working with a database or programming environment
* a map interface to see what nearby u-picks or public gardens are growing so users can find proven varieties to fill in harvest windows
* the web UI should be simple enough to be used by typical retiree gardeners
* all of a user's dac ta should be able to be imported/exported in a simple text format

## dvc
* used to store thumbnail images and original pdf versions of articles instead of committing them to git: https://dvc.org/
* gitignored file michael-gdrive-credentials.json is a google service account credential json file. download it from the google cloud gui when making the service account and make sure that account has access to the google drive folder
* see https://dvc.org/doc/user-guide/how-to/setup-google-drive-remote#using-service-accounts
* workflow:
  * `dvc_add.bat` (does `dvc add --glob -R plant_database\**\*.pdf` etc.)
  * `dvc commit`
  * `dvc push`
  * (and also git commit the new .dvc files)

### dvc windows install
* choco install is broken for some reason. same with the .exe. only pip is working
* choco issue: extracting the .tar.gz during install only gives a .tar, not the folder which is needed for the next steps (installer from dec 12 2022 on windows 11)
* .exe issue: https://github.com/iterative/dvc/issues/7949 (Fixed but not available in the .exe installer as of dec 12 2022)
  * pip install dvc
  * pip install dvc[gdrive]
  * pip install pydrive2

# dvc issues
* dec 2022: can't use a simple glob which catches things that are in .dvcignore (I couldn't figure out ignore patterns within globs on windows): https://github.com/iterative/dvc/issues/7633
