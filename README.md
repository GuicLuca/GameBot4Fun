# GameBot4Fun
The GameBot4Fun bot is a small rust bot used on the GameDev4Fun discord to send daily tips
or send message for live announcement or other things like that.

## Features
1. Daily advice


### 1 - Daily tips
> The bot will send a message every day containing advice for the GD4F community. Advice have some category and messages. They may contain link or images.`

**<div style="text-decoration: underline">To manage tips there is the 5 following commands:</div>**

>##### /tips_list [\<str Tags>]:
> This command will show you the list of tips title already created.
> If tags are specified, it will show only tips that have one of these tags.
> 
> The format of tags given as parameter of this command should be lowercase csv value like following with no spaces around coma : tag1,tag2,tag3,...
> 
> **Example of usage :**
> 
> ![img.png](documentation/tips_list_example.png)

* !tips create \<str Content> \<str Tags> : *Create a new tips*
* !tips update \<int Id> \<str NewContent> \<str NewTags>: *Update the tips content and tags*
* !tips delete \<int Id> : *Delete definitively the tips*

