# Page snapshot

```yaml
- generic [ref=e2]:
  - generic [ref=e5]:
    - heading "This site can’t be reached" [level=1] [ref=e6]:
      - generic [ref=e7]: This site can’t be reached
    - paragraph [ref=e8]:
      - strong [ref=e9]: localhost
      - text: refused to connect.
    - generic [ref=e13]:
      - generic [ref=e14]:
        - paragraph [ref=e15]: "Try:"
        - list [ref=e16]:
          - listitem [ref=e17]: Checking the connection
          - listitem [ref=e18]:
            - link "Checking the proxy and the firewall" [ref=e19] [cursor=pointer]:
              - /url: "#buttons"
      - generic [ref=e20]: ERR_CONNECTION_REFUSED
  - generic [ref=e21]:
    - button "Reload" [ref=e23] [cursor=pointer]
    - button "Details" [ref=e24] [cursor=pointer]
```