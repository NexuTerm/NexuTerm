#include <rfb/rfbclient.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
typedef struct {rfbClient *client;char *password;char *username;uint8_t *pixels;int dirty;char *clipboard;} nx_vnc;
static char tag=0;
static nx_vnc *ctx(rfbClient *c){return rfbClientGetClientData(c,&tag);}
static char *password(rfbClient *c){return strdup(ctx(c)->password);}
static rfbCredential *credentials(rfbClient *c,int type){
    if(type!=rfbCredentialTypeUser)return NULL;
    rfbCredential *cred=calloc(1,sizeof(*cred));if(!cred)return NULL;
    cred->userCredential.username=strdup(ctx(c)->username);
    cred->userCredential.password=strdup(ctx(c)->password);
    return cred;
}
static void wipe(char *text){if(!text)return;volatile char *p=text;size_t n=strlen(text);while(n--)*p++=0;}
static rfbBool allocate(rfbClient *c){
    if(c->width<1||c->height<1||c->width>65535||c->height>65535||(uint64_t)c->width*c->height>16777216)return FALSE;
    free(c->frameBuffer);c->frameBuffer=calloc((size_t)c->width*c->height,4);ctx(c)->pixels=c->frameBuffer;return c->frameBuffer!=NULL;
}
static void update(rfbClient *c,int x,int y,int w,int h){(void)x;(void)y;(void)w;(void)h;ctx(c)->dirty=1;}
static void cut(rfbClient *c,const char *text,int len){if(len<0||len>1048576)return;nx_vnc *n=ctx(c);free(n->clipboard);n->clipboard=malloc((size_t)len+1);if(n->clipboard){memcpy(n->clipboard,text,len);n->clipboard[len]=0;}}
void *nx_vnc_open(const char *host,int port,const char *user,const char *pass){
    nx_vnc *n=calloc(1,sizeof(*n));if(!n)return NULL;
    n->password=strdup(pass);n->username=strdup(user);if(!n->password||!n->username){wipe(n->password);free(n->password);free(n->username);free(n);return NULL;}n->client=rfbGetClient(8,3,4);if(!n->client){wipe(n->password);free(n->password);free(n->username);free(n);return NULL;}
    rfbClient *c=n->client;rfbClientSetClientData(c,&tag,n);c->GetPassword=password;c->GetCredential=credentials;c->MallocFrameBuffer=allocate;c->GotFrameBufferUpdate=update;c->GotXCutText=cut;
    c->serverHost=strdup(host);c->serverPort=port;c->connectTimeout=10;c->readTimeout=10;c->format.bigEndian=FALSE;c->format.redShift=0;c->format.greenShift=8;c->format.blueShift=16;
    c->appData.encodingsString="tight zrle hextile raw";
    if(!rfbInitClient(c,NULL,NULL)){free(n->pixels);wipe(n->password);free(n->password);free(n->username);free(n->clipboard);free(n);return NULL;}wipe(n->password);return n;
}
int nx_vnc_poll(void *opaque){nx_vnc *n=opaque;int r=WaitForMessage(n->client,50000);if(r<0)return -1;if(r>0&&!HandleRFBServerMessage(n->client))return -1;return 0;}
const uint8_t *nx_vnc_frame(void *opaque,int *w,int *h){nx_vnc *n=opaque;if(!n->dirty)return NULL;n->dirty=0;*w=n->client->width;*h=n->client->height;return n->client->frameBuffer;}
void nx_vnc_key(void *opaque,uint32_t key,int down){nx_vnc*n=opaque;SendKeyEvent(n->client,key,down);}
void nx_vnc_pointer(void *opaque,int x,int y,int buttons){nx_vnc*n=opaque;SendPointerEvent(n->client,x,y,buttons);}
void nx_vnc_clipboard(void *opaque,const char *text,int len){nx_vnc*n=opaque;SendClientCutText(n->client,(char*)text,len);}
const char *nx_vnc_take_clipboard(void *opaque){nx_vnc*n=opaque;return n->clipboard;}
void nx_vnc_clear_clipboard(void *opaque){nx_vnc*n=opaque;free(n->clipboard);n->clipboard=NULL;}
void nx_vnc_close(void *opaque){if(!opaque)return;nx_vnc*n=opaque;free(n->client->frameBuffer);n->client->frameBuffer=NULL;rfbClientCleanup(n->client);wipe(n->password);free(n->password);free(n->username);free(n->clipboard);free(n);}
