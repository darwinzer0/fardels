import * as C from '../Const'

const initialState = {
  // account
  keplrEnabled: false,
  scrtAuthorized: false,
  secretClient: null,

  viewingKey: '',
  loggedIn: false,

  // profile   
  //handle: 'zookaxynackle',
  handle: '',
  //profileDescription: 'Zippy forest gnome',
  profileDescription: '',
  //profileThumbnail: '',
  profileThumbnail: '',

  // handle search field
  handleSearchFieldValue: '',

  // Menu
  drawerMenuSelection: C.DRAWER_MENU_MY_FARDELS,

  // Fardel view dialog
  fardelViewDialogOpen: false,

  // View handle
  //viewedHandle: 'zookaxynackle',
  viewedHandle: '',
  //viewedHandleImg: '',
  viewedHandleImg: '',
  viewedFardels: [],
  selectedFardel: -1,

  // Following
  following: [],
  //following: [
  //  'zookaxynackle',
  //],

  // Carry Fardel
  newFardelMessage: '',
  newFardelContents: { 
    contentsText: '', 
    cost: '', 
    ipfsCid: '', 
    passphrase: '', 
  },
  newFardelThumbnail: '',

};
export default initialState;
