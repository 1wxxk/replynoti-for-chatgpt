import React, { useEffect, useState } from 'react';
import {createRoot} from 'react-dom/client';
import {invoke} from '@tauri-apps/api/core';
import {open} from '@tauri-apps/plugin-dialog';
import * as Switch from '@radix-ui/react-switch';
import * as SelectPrimitive from '@radix-ui/react-select';
import {Bell, BellRing, AudioLines, Palette, Settings2, ChevronRight, ChevronDown, Check, Monitor, Moon, Sun, Image, FolderOpen, RotateCcw, Play, Globe2, Info, CircleHelp, ArrowUpRight, ShieldCheck, LoaderCircle, X} from 'lucide-react';
import './style.css';

type Pref={display_name:string;enabled:boolean;notification_mode:string;idle_seconds:number;preview:boolean;length:number;navigate:boolean;sound:string;custom_icon:boolean;startup:boolean;language:string;theme:string};
type State={preferences:Pref;icon_data:string;has_sound:boolean;has_icon:boolean};
type Tab='notifications'|'sound'|'appearance'|'general';
const initial:Pref={display_name:'ReplyNoti',enabled:true,notification_mode:"smart",idle_seconds:10,preview:true,length:100,navigate:true,sound:'default',custom_icon:false,startup:false,language:'system',theme:'system'};
const demo:State={preferences:initial,icon_data:'/replynoti.png',has_sound:false,has_icon:false};
const native=typeof window!=='undefined'&&'__TAURI_INTERNALS__' in window;
const dict={
 ko:{notifyMode:'알림 조건',notifyModeDesc:'답변 완료 알림을 받을 상황을 선택합니다.',smart:'스마트 알림 (권장)',always:'항상 알림',background:'백그라운드에서만',idleTime:'자리 비움 기준',idleDesc:'ChatGPT 창이 활성화된 경우 마지막 PC 입력 이후 이 시간이 지나면 보류된 알림을 보냅니다.',settings:'설정',tagline:'답변이 준비되면, 알려드릴게요.',notifications:'알림',sound:'알림음',appearance:'외관',general:'일반',running:'알림 감지 활성화',paused:'알림 감지 일시 중지',notifyTitle:'알림 설정',notifyDesc:'답변 생성이 끝났을 때 받을 알림을 설정하세요.',complete:'답변 완료 알림',completeDesc:'ChatGPT 답변 생성이 끝나면 알려드립니다.',preview:'답변 미리보기',previewDesc:'알림에 완료된 답변의 일부를 표시합니다.',length:'미리보기 길이',lengthDesc:'알림에 표시할 최대 글자 수입니다.',click:'알림 클릭 동작',clickDesc:'알림을 눌렀을 때 해당 대화로 이동합니다.',privacy:'미리보기는 잠금 화면과 알림 기록에 남을 수 있습니다.',test:'테스트 알림',sample:'알림 미리보기',now:'지금',sampleText:'답변이 준비되었습니다. 이제 다른 작업을 마치고 확인해 보세요.',generic:'ChatGPT 답변 생성이 완료되었습니다.',unofficial:'독립적인 비공식 Windows 도우미',soundTitle:'알림음 설정',soundDesc:'답변 완료 시 재생할 소리를 선택하세요.',defaultSound:'Windows 기본음',customSound:'사용자 지정 WAV',silent:'무음',soundChoice:'재생할 알림음',soundHint:'Windows 기본 알림음 또는 원하는 음원을 사용하세요.',importSound:'설치된 앱 완료음 가져오기',chooseSound:'WAV 파일 선택',soundReady:'사용자 지정 알림음이 준비되었습니다.',soundMissing:'아직 사용자 지정 음원이 없습니다.',soundPrivacy:'설치된 앱에서 가져온 음원은 PC에만 저장됩니다. 사용자 지정 음원은 Windows 방해 금지 설정과 다르게 동작할 수 있습니다.',appearanceTitle:'외관 설정',appearanceDesc:'알림에 표시되는 이름과 아이콘을 나에게 맞게 바꾸세요.',display:'표시 이름',displayDesc:'알림 제목과 트레이 이름을 변경합니다.',icon:'알림 아이콘',iconDesc:'트레이 및 새 Windows 알림의 아이콘입니다.',importIcon:'설치된 앱 아이콘 가져오기',chooseIcon:'PNG / ICO 선택',resetIcon:'기본 아이콘으로 복원',theme:'테마',themeDesc:'설정 화면의 색상 모드를 선택합니다.',system:'시스템',light:'라이트',dark:'다크',generalTitle:'일반 설정',generalDesc:'시작 동작과 인터페이스 언어를 관리합니다.',startup:'Windows 시작 시 실행',startupDesc:'로그인할 때 ReplyNoti을 자동으로 실행합니다.',language:'언어',languageDesc:'설정 화면에서 사용할 언어입니다.',about:'ReplyNoti 정보',aboutDesc:'답변 완료를 알려주는 가벼운 Windows 알림 도우미입니다.',privacyTitle:'로컬 우선',privacyDesc:'답변 미리보기와 사용자 지정 리소스는 이 PC에서만 처리합니다. 공식 서비스와 제휴하지 않습니다.',save:'변경사항 저장',saved:'저장되었습니다',dirty:'저장하지 않은 변경사항',discard:'변경 취소',ready:'모든 설정이 저장되었습니다',loading:'설정을 불러오는 중…',error:'설정을 처리하지 못했습니다.',imported:'리소스를 가져왔습니다.',sent:'테스트 알림을 요청했습니다.',defaultIcon:'기본 아이콘',customIcon:'사용자 지정 아이콘',confirm:'설치된 앱의 리소스를 개인 환경에서 가져올까요? 사용 권한이 있는 리소스만 사용하세요.',cancel:'취소',import:'가져오기',testMode:'미리보기 모드',testModeDesc:'브라우저 시안에서는 실제 PC 설정이 변경되지 않습니다.'},
 en:{notifyMode:'Notify when',notifyModeDesc:'Choose when to receive completion notifications.',smart:'Smart (recommended)',always:'Always',background:'Background only',idleTime:'Idle threshold',idleDesc:'While ChatGPT is focused, defer alerts until this time has passed since your last PC input.',settings:'Settings',tagline:'Know when your answer is ready.',notifications:'Notifications',sound:'Sound',appearance:'Appearance',general:'General',running:'Notifications enabled',paused:'Detection paused',notifyTitle:'Notifications',notifyDesc:'Choose how you hear about completed answers.',complete:'Completion notifications',completeDesc:'Notify me when ChatGPT finishes generating.',preview:'Reply preview',previewDesc:'Include part of a completed reply in the notification.',length:'Preview length',lengthDesc:'Maximum length of the reply excerpt.',click:'Open conversation',clickDesc:'Open the matching conversation when a notification is clicked.',privacy:'Previews can appear on your lock screen and in notification history.',test:'Test notification',sample:'Notification preview',now:'Now',sampleText:'Your answer is ready. Finish what you are doing and take a look.',generic:'A ChatGPT reply is ready.',unofficial:'Independent, unofficial Windows companion',soundTitle:'Notification sound',soundDesc:'Choose what plays when an answer is ready.',defaultSound:'Windows default',customSound:'Custom WAV',silent:'Silent',soundChoice:'Notification sound',soundHint:'Use the Windows notification sound or your own audio.',importSound:'Import installed app sound',chooseSound:'Choose WAV file',soundReady:'A custom sound is ready.',soundMissing:'No custom sound imported yet.',soundPrivacy:'Imported audio stays on this PC. Custom audio may not follow Windows Do Not Disturb.',appearanceTitle:'Appearance',appearanceDesc:'Make the notification name and icon your own.',display:'Display name',displayDesc:'Changes toast titles and the tray label.',icon:'Notification icon',iconDesc:'Artwork used for the tray and future Windows notifications.',importIcon:'Import installed app icon',chooseIcon:'Choose PNG / ICO',resetIcon:'Restore default icon',theme:'Theme',themeDesc:'Choose a color theme for the settings window.',system:'System',light:'Light',dark:'Dark',generalTitle:'General',generalDesc:'Manage startup behavior and interface language.',startup:'Start with Windows',startupDesc:'Launch ReplyNoti automatically when you sign in.',language:'Language',languageDesc:'Choose the language of this settings window.',about:'About ReplyNoti',aboutDesc:'A lightweight Windows companion for reply completion notifications.',privacyTitle:'Local first',privacyDesc:'Reply previews and imported assets are processed on this PC. Not affiliated with the official service.',save:'Save changes',saved:'Settings saved',dirty:'Unsaved changes',discard:'Discard changes',ready:'All settings saved',loading:'Loading settings…',error:'Unable to update settings.',imported:'Resource imported.',sent:'Test notification requested.',defaultIcon:'Default icon',customIcon:'Custom icon',confirm:'Import a resource from the installed app for local use? Use only assets you have permission to use.',cancel:'Cancel',import:'Import',testMode:'Preview mode',testModeDesc:'This browser preview does not modify your PC settings.'}
};
type Choice={v:string;l:string};
type DropdownProps={label:string;value:string;items:Choice[];onChange:(v:string)=>void;disabled?:boolean;compact?:boolean};
function Dropdown({label,value,items,onChange,disabled=false,compact=false}:DropdownProps){
 return <SelectPrimitive.Root value={value} onValueChange={onChange} disabled={disabled}>
  <SelectPrimitive.Trigger className={'dropdown-trigger'+(compact?' compact':'')} aria-label={label}>
   <SelectPrimitive.Value/><SelectPrimitive.Icon asChild><ChevronDown size={15} className="dropdown-chevron"/></SelectPrimitive.Icon>
  </SelectPrimitive.Trigger>
  <SelectPrimitive.Portal>
   <SelectPrimitive.Content className="dropdown-content" position="popper" sideOffset={6} collisionPadding={12} align="end">
    <SelectPrimitive.Viewport className="dropdown-viewport">
     {items.map(item=><SelectPrimitive.Item className="dropdown-item" value={item.v} key={item.v}><SelectPrimitive.ItemText>{item.l}</SelectPrimitive.ItemText><SelectPrimitive.ItemIndicator asChild><Check size={15}/></SelectPrimitive.ItemIndicator></SelectPrimitive.Item>)}
    </SelectPrimitive.Viewport>
   </SelectPrimitive.Content>
  </SelectPrimitive.Portal>
 </SelectPrimitive.Root>;
}
function Toggle({label,desc,checked,onChange,disabled=false,busy=false}:{label:string;desc:string;checked:boolean;onChange:(b:boolean)=>void;disabled?:boolean;busy?:boolean}){
 return <div className={'setting-row'+(disabled?' disabled':'')}><div><strong>{label}</strong><p>{desc}</p></div><Switch.Root className="switch" checked={checked} onCheckedChange={onChange} disabled={disabled||busy} aria-label={label}><Switch.Thumb className="thumb"/></Switch.Root></div>;
}
function Select({label,desc,value,items,onChange,disabled=false,busy=false}:DropdownProps&{desc:string;busy?:boolean}){
 return <div className={'setting-row'+(disabled?' disabled':'')}><div><strong>{label}</strong><p>{desc}</p></div><Dropdown label={label} value={value} items={items} onChange={onChange} disabled={disabled||busy}/></div>;
}

function App(){
 const [state,setState]=useState<State>(demo),[p,setP]=useState<Pref>(initial),[tab,setTab]=useState<Tab>('notifications'),[busy,setBusy]=useState(false),[loading,setLoading]=useState(native),[message,setMessage]=useState(''),[error,setError]=useState(false),[confirm,setConfirm]=useState<'icon'|'sound'|null>(null);
 const [systemDark,setSystemDark]=useState(matchMedia('(prefers-color-scheme: dark)').matches);
 const lang=p.language==='ko'||(p.language==='system'&&navigator.language.toLowerCase().startsWith('ko'))?'ko':'en';
 const t=dict[lang];const dark=p.theme==='dark'||(p.theme==='system'&&systemDark);
 const dirty=JSON.stringify(p)!==JSON.stringify(state.preferences);
 useEffect(()=>{const m=matchMedia('(prefers-color-scheme: dark)');const listener=()=>setSystemDark(m.matches);m.addEventListener('change',listener);return()=>m.removeEventListener('change',listener)},[]);
 useEffect(()=>{document.documentElement.dataset.theme=dark?'dark':'light';document.documentElement.lang=lang;},[dark,lang]);
 useEffect(()=>{if(native)invoke<State>('load_settings').then(s=>{setState(s);setP(s.preferences)}).catch(e=>{setMessage(String(e));setError(true)}).finally(()=>setLoading(false));else{setP({...initial,language:'ko',theme:'dark'});setState({...demo,preferences:{...initial,language:'ko',theme:'dark'}})}},[]);
 const edit=<K extends keyof Pref,>(key:K,value:Pref[K])=>setP(old=>({...old,[key]:value}));
 function notice(m:string,isError=false){setMessage(m);setError(isError)}
 async function save(test=false){
  if(busy||loading)return;
  setBusy(true);setMessage('');
  try{
   let saved:State;
   if(native)saved=await invoke<State>('save_settings',{preferences:p});
   else saved={...state,preferences:{...p}};
   setState(saved);setP(saved.preferences);
   if(test&&native)await invoke('test_notification');
   notice(test?t.sent:t.saved);
  }catch(e){notice(t.error+' '+String(e),true)}finally{setBusy(false)}
 }
 async function importFile(kind:'icon'|'sound',installed=false){
  if(busy)return;setConfirm(null);setBusy(true);
  try{
   if(!native){notice(t.testModeDesc);return;}
   let source:string|null=null;
   if(!installed){const path=await open({multiple:false,directory:false,filters:[kind==='icon'?{name:'PNG / ICO',extensions:['png','ico']}:{name:'WAV',extensions:['wav']}]});if(typeof path!=='string')return;source=path;}
   const next=await invoke<State>('import_asset',{kind,source});
   setState(next);setP(old=>({...old,...(kind==='icon'?{custom_icon:true}:{sound:'custom'})}));notice(t.imported);
  }catch(e){notice(t.error+' '+String(e),true)}finally{setBusy(false)}
 }
 const currentIcon=p.custom_icon?state.icon_data:'/replynoti.png';
 const nav=[{id:'notifications',label:t.notifications,Icon:Bell},{id:'sound',label:t.sound,Icon:AudioLines},{id:'appearance',label:t.appearance,Icon:Palette},{id:'general',label:t.general,Icon:Settings2}] as const;
 const Title=({title,desc}:{title:string,desc:string})=><header className="page-heading"><h1>{title}</h1><p>{desc}</p></header>;
 const Action=({children,onClick,icon:Icon,primary=false,disabled=false}:{children:React.ReactNode,onClick:()=>void,icon?:React.ElementType,primary?:boolean,disabled?:boolean})=><button type="button" className={primary?'button primary':'button'} disabled={disabled||busy} onClick={onClick}>{Icon&&<Icon size={16} aria-hidden="true"/>}{children}</button>;
 const Preview=({test=false}:{test?:boolean})=> <div className="preview-area"><div className="preview-heading"><div className="section-label"><BellRing size={15}/>{t.sample}</div>{test&&<Action icon={BellRing} onClick={()=>void save(true)}>{t.test}</Action>}</div><div className="toast-preview"><div className="toast-top"><img src={currentIcon} alt="" onError={e=>{e.currentTarget.src='/replynoti.png'}}/><div><b>{p.display_name||'ReplyNoti'}</b><span>{t.now}</span></div><X size={15}/></div><p>{p.preview?t.sampleText:t.generic}</p></div><div className="preview-foot"><Info size={14}/>{t.privacy}</div></div>;
 return <div className="shell">
  <aside className="sidebar">
   <div className="brand"><img src="/replynoti.png" alt=""/><div><b>ReplyNoti</b><span>WINDOWS COMPANION</span></div></div>
   <div className="nav-label">{t.settings}</div>
   <nav aria-label={t.settings}>{nav.map(({id,label,Icon})=><button key={id} aria-current={tab===id?'page':undefined} className={tab===id?'nav active':'nav'} onClick={()=>{setTab(id);setMessage('')}}><Icon size={19}/><span>{label}</span>{tab===id&&<ChevronRight className="nav-arrow" size={16}/>}</button>)}</nav>
   <div className="sidebar-bottom"><div className="status"><span className={'dot '+(p.enabled?'on':'')}/><span>{p.enabled?t.running:t.paused}</span></div><div className="version">v0.1.0-beta.7 <span>·</span> {t.unofficial}</div></div>
  </aside>
  <div className="workspace"><header className="topbar"><span>{t.settings} <ChevronRight size={14}/> <b>{nav.find(n=>n.id===tab)?.label}</b></span><div className="quick"><Globe2 size={16}/><Dropdown compact label={t.language} value={p.language} items={[{v:'system',l:t.system},{v:'ko',l:'한국어'},{v:'en',l:'English'}]} onChange={v=>edit('language',v)} disabled={busy}/></div></header>
   <main className="content" aria-busy={loading||busy}>
   {loading?<div className="loading"><LoaderCircle className="spin" size={25}/>{t.loading}</div>:<>
    {!native&&<div className="demo-note"><Info size={16}/>{t.testModeDesc}</div>}
    {tab==='notifications'&&<><Title title={t.notifyTitle} desc={t.notifyDesc}/><div className="group"><Toggle busy={busy} label={t.complete} desc={t.completeDesc} checked={p.enabled} onChange={b=>edit('enabled',b)}/><Select busy={busy} label={t.notifyMode} desc={t.notifyModeDesc} value={p.notification_mode} items={[{v:"smart",l:t.smart},{v:"always",l:t.always},{v:"background",l:t.background}]} onChange={v=>edit("notification_mode",v)} disabled={!p.enabled}/>{p.notification_mode==="smart"&&<Select busy={busy} label={t.idleTime} desc={t.idleDesc} value={String(p.idle_seconds)} items={[10,30,60,180].map(v=>({v:String(v),l:lang==="ko"?(v<60?v+"초":v/60+"분"):(v<60?v+" sec":v/60+" min")}))} onChange={v=>edit("idle_seconds",Number(v))} disabled={!p.enabled}/>}<Toggle busy={busy} label={t.preview} desc={t.previewDesc} checked={p.preview} onChange={b=>edit('preview',b)}/><Select busy={busy} label={t.length} desc={t.lengthDesc} value={String(p.length)} items={[50,100,150].map(v=>({v:String(v),l:lang==='ko'?v+'자':v+' characters'}))} onChange={v=>edit('length',Number(v))} disabled={!p.preview}/><Toggle busy={busy} label={t.click} desc={t.clickDesc} checked={p.navigate} onChange={b=>edit('navigate',b)}/></div><Preview test/></>}
    {tab==='sound'&&<><Title title={t.soundTitle} desc={t.soundDesc}/><div className="group"><Select busy={busy} label={t.soundChoice} desc={t.soundHint} value={p.sound} items={[{v:'default',l:t.defaultSound},{v:'custom',l:t.customSound},{v:'silent',l:t.silent}]} onChange={v=>edit('sound',v)}/><div className="asset-section"><div className="asset-header"><div className="asset-mark"><AudioLines size={24}/></div><div><strong>{t.customSound}</strong><p>{state.has_sound?t.soundReady:t.soundMissing}</p></div></div><div className="asset-buttons"><Action icon={FolderOpen} onClick={()=>void importFile('sound')}>{t.chooseSound}</Action><Action icon={ArrowUpRight} onClick={()=>setConfirm('sound')}>{t.importSound}</Action></div></div></div><div className="info-box"><ShieldCheck size={19}/><p>{t.soundPrivacy}</p></div><Action icon={Play} onClick={()=>void save(true)}>{t.test}</Action></>}
    {tab==='appearance'&&<><Title title={t.appearanceTitle} desc={t.appearanceDesc}/><div className="group"><div className="setting-row"><div><label htmlFor="name"><strong>{t.display}</strong></label><p>{t.displayDesc}</p></div><input id="name" className="name-input" value={p.display_name} maxLength={48} onChange={e=>edit('display_name',e.target.value)} disabled={busy}/></div><div className="asset-section"><div className="asset-header"><img className="icon-sample" src={currentIcon} alt="" onError={e=>e.currentTarget.src='/replynoti.png'}/><div><strong>{t.icon}</strong><p>{t.iconDesc}</p><span className="asset-caption">{p.custom_icon?t.customIcon:t.defaultIcon}</span></div></div><div className="asset-buttons"><Action icon={Image} onClick={()=>void importFile('icon')}>{t.chooseIcon}</Action><Action icon={ArrowUpRight} onClick={()=>setConfirm('icon')}>{t.importIcon}</Action><Action icon={RotateCcw} disabled={!p.custom_icon} onClick={()=>edit('custom_icon',false)}>{t.resetIcon}</Action></div></div><div className="setting-row"><div><strong>{t.theme}</strong><p>{t.themeDesc}</p></div><div className="segmented" role="group" aria-label={t.theme}>{([{v:'system',l:t.system,I:Monitor},{v:'light',l:t.light,I:Sun},{v:'dark',l:t.dark,I:Moon}]).map(({v,l,I})=><button key={v} aria-pressed={p.theme===v} onClick={()=>edit('theme',v)} className={p.theme===v?'selected':''}><I size={15}/>{l}</button>)}</div></div></div><Preview/></>}
    {tab==='general'&&<><Title title={t.generalTitle} desc={t.generalDesc}/><div className="group"><Toggle busy={busy} label={t.startup} desc={t.startupDesc} checked={p.startup} onChange={b=>edit('startup',b)}/><Select busy={busy} label={t.language} desc={t.languageDesc} value={p.language} items={[{v:'system',l:t.system},{v:'ko',l:'한국어'},{v:'en',l:'English'}]} onChange={v=>edit('language',v)}/></div><section className="about"><div className="asset-header"><img src="/replynoti.png" alt="" className="icon-sample"/><div><h2>{t.about}</h2><p>v0.1.0-beta.7 · MIT</p></div></div><p>{t.aboutDesc}</p><div className="info-box"><ShieldCheck size={19}/><div><strong>{t.privacyTitle}</strong><p>{t.privacyDesc}</p></div></div></section></>}
   </>}
   </main>
   <footer className="footer"><div className="save-status" aria-live="polite">{message?<span className={error?'error':'notice'}>{message}</span>:<span>{dirty?<span className="unsaved-dot"/>:<Check size={15}/>} {dirty?t.dirty:t.ready}</span>}</div><div className="footer-buttons">{dirty&&<Action onClick={()=>{setP(state.preferences);setMessage('')}}>{t.discard}</Action>}<Action primary disabled={!dirty||loading} icon={busy?LoaderCircle:Check} onClick={()=>void save()}>{t.save}</Action></div></footer>
  </div>
  {confirm&&<div className="overlay" onClick={()=>setConfirm(null)}><div className="dialog" role="dialog" aria-modal="true" aria-label={confirm==='icon'?t.importIcon:t.importSound} onClick={e=>e.stopPropagation()}><h2>{confirm==='icon'?t.importIcon:t.importSound}</h2><p>{t.confirm}</p><div className="footer-buttons"><Action onClick={()=>setConfirm(null)}>{t.cancel}</Action><Action primary onClick={()=>void importFile(confirm,true)}>{t.import}</Action></div></div></div>}
 </div>
}
createRoot(document.getElementById('root')!).render(<App/>);
